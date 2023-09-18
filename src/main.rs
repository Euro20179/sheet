mod calculator;
mod position_parser;
mod sheet_tokenizer;
mod table;

use std::io::{Read, Stdin};

use base64::{engine, prelude::*};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use table::Table;

use crate::table::Direction;

#[derive(Eq, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

struct KeySequence {
    pub count: usize,
    pub action: char,
    pub key: String
}

struct Program<'a> {
    mode: Mode,
    file_path: String,
    table: &'a mut Table,
}

fn handle_normal_mode(program: &mut Program, key: KeySequence) {
    let table = &mut program.table;
    //TODO: undo mode
    //table could keep track of previous instances of rows/columns
    //when u is pressed it restores the previous instance of rows/columns
    match key.key.as_str() {
        "i" => program.mode = Mode::Insert,
        "y" => {
            let data = table.get_value_at_position(&table.get_pos());
            let s = match data {
                table::Data::Number(n) | table::Data::Equation(n, ..) | table::Data::String(n) => n,
            };
            let encoded = engine::general_purpose::STANDARD.encode(s);
            print!("\x1b]52;c;{}\x07", encoded)
        }
        "s" => {
            table.clear_cell(&table.get_pos());
            program.mode = Mode::Insert
        }
        "\x1b[B" | "j" => {
            for _ in 0..key.count {
                if table.cursor_at_bottom() {
                    table.add_row(table.get_pos().row + 1);
                }
                table.move_cursor(Direction::Down);
            }
        }
        "\x1b[A" | "k" => {
            for _ in 0..key.count {
                table.move_cursor(Direction::Up);
            }
        }
        "\x1b[C" | "l" => {
            for _ in 0..key.count {
                table.move_cursor(Direction::Right);
            }
        }
        "L" => table.move_cursor(Direction::MostRight),
        "H" => table.move_cursor(Direction::MostLeft),
        "\x1b[D" | "h" => {
            for _ in 0..key.count {
                table.move_cursor(Direction::Left);
            }
        }
        "g" | "K" => table.move_cursor(Direction::Top),
        "G" | "J" => table.move_cursor(Direction::Bottom),
        "R" => {
            let pos = table.get_pos();
            for _ in 0..key.count {
                table.add_row(pos.row);
            }
        }
        "r" => {
            let pos = table.get_pos();
            for _ in 0..key.count {
                table.add_row(pos.row + 1);
                table.move_cursor(Direction::Down);
            }
        }
        "c" => {
            let pos = table.get_pos();
            for _ in 0..key.count {
                table.add_col(pos.col + 1);
                table.move_cursor(Direction::Right)
            }
        }
        "C" => {
            let pos = table.get_pos();
            for _ in 0..key.count {
                table.add_col(pos.col);
            }
        }
        "d" => {
            let row = table.get_pos().row;
            table.remove_row(row);
        }
        "D" => {
            let col = table.get_pos().col;
            table.remove_col(col);
        }
        "w" => {
            let sheet = table.to_sheet();
            std::fs::write(&program.file_path, sheet).unwrap();
        }
        "x" => {
            let pos = table.get_pos();
            table.clear_cell(&pos)
        }
        _ => {}
    }
}

fn handle_insert_mode(program: &mut Program, key: KeySequence) {
    let table = &mut program.table;
    match key.action as u8 {
        //backspace
        127 => table.remove_last_char_in_cell(&table.get_pos()),
        10 => program.mode = Mode::Normal,
        b'\t' => {
            table.move_cursor(Direction::Right)
        }
        b'=' => {
            if table.cursor_pos_is_empty() {
                table.convert_cell(&table.get_pos(), table::Data::Equation(String::new(), None))
            } else {
                table.append_text_to_cell(&table.get_pos(), key.key);
            }
        }
        _ => table.append_text_to_cell(&table.get_pos(), key.key),
    }
}

fn handle_mode(program: &mut Program, key: KeySequence) {
    match program.mode {
        Mode::Normal => handle_normal_mode(program, key),
        Mode::Insert => handle_insert_mode(program, key),
    }
}

fn get_key(program: &Program, reader: &mut Stdin) -> KeySequence {
    let mut count = String::new();
    let mut buf = [0; 32]; //consume enough bytes to store utf-8, 32 bytes should be enough
    loop {
        let bytes_read = reader.read(&mut buf).unwrap();
        let key = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();
        let ch = buf[0];

        if ch >= 48 && ch <= 57 && program.mode == Mode::Normal {
            count += &String::from(ch as char);
        } else {
            if count == "" {
                count = String::from("1");
            }
            return KeySequence {
                action: ch as char,
                count: count.parse().unwrap(),
                key
            };
        }
    }
}

fn main() {
    // let mut lexer = calculator::Lexer::new("sum($a1:$b1)/2".to_string());
    // let toks = lexer.tokenize();
    // println!("{:?}", toks);
    // let mut parser = calculator::Parser::new(toks);
    // let tree = parser.build_tree();
    // println!("{:?}", tree);
    // let int = calculator::Interpreter::new(tree);
    // let symbols: HashMap<String, calculator::Result> = HashMap::new();
    // println!(
    //     "{:?}",
    //     int.interpret(
    //         &symbols,
    //         &Table::from_tokens(vec![
    //             sheet_tokenizer::Token::LBracket,
    //             sheet_tokenizer::Token::Number(3.0),
    //             sheet_tokenizer::Token::Number(3.0),
    //             sheet_tokenizer::Token::RBracket
    //         ])
    //     )
    // );

    // let table = Table::from_tokens(vec![
    //     sheet_tokenizer::Token::LBracket,
    //     sheet_tokenizer::Token::Number(0.3),
    //     sheet_tokenizer::Token::RBracket
    // ]);
    //
    // println!("{}", base_10_to_col_num(10));
    //
    // table.display(10, true);

    let mut args = std::env::args();

    let mut file_name: Option<String> = None;
    if args.len() > 1 {
        file_name = args.nth(1);
    }

    if let None = file_name {
        eprintln!("No file name given");
    }

    let fp = file_name.unwrap();

    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios.clone();
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[termios::VMIN] = 1;
    new_termios.c_cc[termios::VTIME] = 10;
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

    let data = std::fs::read_to_string(&fp);
    let mut text = String::from("[]");
    if let Ok(t) = data {
        text = t;
    }


    let toks = sheet_tokenizer::parse(text.as_str());

    let mut table = Table::from_tokens(toks);

    let mut program = Program {
        table: &mut table,
        file_path: fp,
        mode: Mode::Normal,
    };

    let mut reader = std::io::stdin();

    loop {
        print!("\x1b[2J\x1b[0H");
        let do_equations = match program.mode {
            Mode::Insert => false,
            _ => true,
        };
        program.table.display(10, do_equations);
        //TODO: detect multi-char keys eg: ^[A
        let key_sequence = get_key(&program, &mut reader);
        //TODO: add detection for if the file is saved
        if key_sequence.action == 'q' && program.mode == Mode::Normal {
            break;
        } else {
            handle_mode(&mut program, key_sequence);
        }
    }

    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}
