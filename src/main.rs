mod calculator;
mod position_parser;
mod sheet_tokenizer;
mod table;

use std::io::Read;

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use table::Table;

use crate::table::Direction;

#[derive(Eq, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

struct Program<'a> {
    mode: Mode,
    file_path: String,
    table: &'a mut Table
}

fn handle_normal_mode(program: &mut Program, key: u8) {
    let table = &mut program.table;
    match key {
        b'i' => program.mode = Mode::Insert,
        b's' => {
            table.clear_cell(&table.get_pos());
            program.mode = Mode::Insert
        }
        b'j' => table.move_cursor(Direction::Down),
        b'k' => table.move_cursor(Direction::Up),
        b'l' => table.move_cursor(Direction::Right),
        b'L' => table.move_cursor(Direction::MostRight),
        b'H' => table.move_cursor(Direction::MostLeft),
        b'h' => table.move_cursor(Direction::Left),
        b'g' | b'K' => table.move_cursor(Direction::Top),
        b'G' | b'J' => table.move_cursor(Direction::Bottom),
        b'R' => {
            let pos = table.get_pos();
            table.add_row(pos.row)
        }
        b'r' => {
            let pos = table.get_pos();
            table.add_row(pos.row + 1)
        }
        b'c' => {
            let pos = table.get_pos();
            table.add_col(pos.col + 1);
        }
        b'C' => {
            let pos = table.get_pos();
            table.add_col(pos.col);
        }
        b'd' => {
            let row = table.get_pos().row;
            table.remove_row(row);
        }
        b'D' => {
            let col = table.get_pos().col;
            table.remove_col(col);
        }
        b'w' => {
            let sheet = table.to_sheet();
            std::fs::write(&program.file_path, sheet).unwrap();
        }
        b'x' => {
            let pos = table.get_pos();
            table.clear_cell(&pos)
        }
        _ => {}
    }
}

fn handle_insert_mode(program: &mut Program, key: u8) {
    let table = &mut program.table;
    eprintln!("{}", key);
    match key {
        //backspace
        127 => table.remove_last_char_in_cell(&table.get_pos()),
        10 => {
            program.mode = Mode::Normal
        }, 
        b'=' => {
            if table.cursor_pos_is_empty() {
                table.convert_cell(&table.get_pos(), table::Data::Equation(String::new(), None))
            }
            else {
                table.append_char_to_cell(&table.get_pos(), key as char);
            }
        },
        _ => table.append_char_to_cell(&table.get_pos(), key as char)

    }
}

fn handle_mode(program: &mut Program, key: u8) {
    match program.mode {
        Mode::Normal => handle_normal_mode(program, key),
        Mode::Insert => handle_insert_mode(program, key),
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
        mode: Mode::Normal
    };

    let mut reader = std::io::stdin();
    loop {
        print!("\x1b[2J\x1b[0H");
        let do_equations = match program.mode {
            Mode::Insert => false,
            _ => true
        };
        program.table.display(10, do_equations);
        let mut buf = [0; 1];
        reader.read_exact(&mut buf).unwrap();
        let ch = buf[0];
        if ch == b'q' && program.mode == Mode::Normal {
            break;
        } else {
            handle_mode(&mut program, ch);
        }
    }

    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}
