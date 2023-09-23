mod calculator;
mod command_line;
mod position_parser;
mod program;
mod sheet_tokenizer;
mod table;
use command_line::CommandLine;
use program::Program;
use std::os::unix::io::AsRawFd;
use table::{Direction, Position, Table};

use std::{
    env::Args,
    io::{BufRead, Read, Stdin},
};

use base64::{engine, prelude::*};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

struct ProgramArguments {
    opts: std::collections::HashMap<String, String>,
    file: String,
}

fn parse_args(args: &mut Args) -> ProgramArguments {
    let _prog_name = args.next(); //skip prog_name
    let mut opts: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut file_name: Option<String> = None;
    let mut parsing_opts = true;
    while let Some(s) = args.next() {
        if s == "--" {
            parsing_opts = false;
        } else if parsing_opts && s.starts_with("-") {
            match args.next() {
                Some(v) => opts.insert(s, v),
                None => break,
            };
        } else {
            file_name = Some(s);
            break;
        }
    }
    match file_name {
        None => ProgramArguments {
            opts,
            file: "-".to_string(),
        },
        Some(file) => ProgramArguments { opts, file },
    }
}

fn execute_command(program: &mut program::Program, command: &str) {
    if command == "q" {
        std::process::exit(0);
    }
    if command == "w" {
        let sheet = program.table.to_sheet();
        std::fs::write(program.get_file_path(), sheet).unwrap();
        program.command_line.print("Saved");
    }
}

fn handle_command_mode(program: &mut program::Program, key: program::KeySequence) {
    if key.action as u8 == 10 {
        let text = program.command_line.get_current_text().to_owned();
        program.command_line.clear_text();
        execute_command(program, &text);
        program.set_mode(program::Mode::Normal);
    } else if key.action as u8 == 127 {
        program.command_line.remove_last_char();
    } else {
        program.command_line.add_text_to_current_command(&key.key);
    }
}

fn get_range_from_motion(program: &Program, motion: &str) -> (Position, Position) {
    let pos = program.table.get_pos();
    return match motion {
        "l" | "h" => {
            let pos = program.table.get_pos();
            (
                Position {
                    row: pos.row,
                    col: 0,
                },
                Position {
                    row: pos.row,
                    col: program.table.get_size()[1],
                },
            )
        }
        "j" | "k" => {
            let pos = program.table.get_pos();
            return (
                Position {
                    row: 0,
                    col: pos.col,
                },
                Position {
                    row: program.table.get_size()[0],
                    col: 0,
                },
            );
        },
        _ => return (pos, pos)
    };
}

fn handle_normal_mode(program: &mut program::Program, key: program::KeySequence) {
    //TODO: undo mode
    //table could keep track of previous instances of rows/columns
    //when u is pressed it restores the previous instance of rows/columns
    match key.key.as_str() {
        //TODO: add detection for if the file is saved
        "q" => program.running = false,
        "i" => program.set_mode(program::Mode::Insert),
        ":" => {
            program.command_line.clear_text();
            program.set_mode(program::Mode::Command);
        }
        ">" => {
            let pos = program.table.get_pos();
            let cur_size = program.table.get_col_width(pos.col).unwrap();
            program.table.resize_col(pos.col, cur_size + 1);
        }
        "<" => {
            let pos = program.table.get_pos();
            let cur_size = program.table.get_col_width(pos.col).unwrap();
            if cur_size > 0 {
                program.table.resize_col(pos.col, cur_size - 1);
            }
        }
        "y" => {
            let mut reader = std::io::stdin();
            let direction = program.get_key(&mut reader);
            let mut s: String = String::new();
            let range = get_range_from_motion(program, &direction.action.to_string());
            let data = program.table.get_values_at_range(&range.0, &range.1);
            for d in data {
                let temp = match d {
                            table::Data::Number(n)
                            | table::Data::Equation(n, ..)
                            | table::Data::String(n) => n,
                };
                s += &(temp + &String::from("\t"))
            }
            let encoded = engine::general_purpose::STANDARD.encode(s);
            print!("\x1b]52;c;{}\x07", encoded)
        }
        "s" => {
            program.table.clear_cell(&program.table.get_pos());
            program.set_mode(program::Mode::Insert);
        }
        "\x1b[B" | "j" => {
            for _ in 0..key.count {
                if program.table.cursor_at_bottom() {
                    program.table.add_row(program.table.get_pos().row + 1);
                }
                program.table.move_cursor(Direction::Down);
            }
        }
        "\x1b[A" | "k" => {
            for _ in 0..key.count {
                program.table.move_cursor(Direction::Up);
            }
        }
        "\x1b[C" | "l" => {
            for _ in 0..key.count {
                if program.table.cursor_at_right() {
                    program.table.add_col(program.table.get_pos().col + 1);
                }
                program.table.move_cursor(Direction::Right);
            }
        }
        "L" => program.table.move_cursor(Direction::MostRight),
        "H" => program.table.move_cursor(Direction::MostLeft),
        "\x1b[D" | "h" => {
            for _ in 0..key.count {
                program.table.move_cursor(Direction::Left);
            }
        }
        "g" | "K" => program.table.move_cursor(Direction::Top),
        "G" | "J" => program.table.move_cursor(Direction::Bottom),
        "R" => {
            let pos = program.table.get_pos();
            for _ in 0..key.count {
                program.table.add_row(pos.row);
            }
        }
        "r" => {
            let pos = program.table.get_pos();
            for _ in 0..key.count {
                program.table.add_row(pos.row + 1);
                program.table.move_cursor(Direction::Down);
            }
        }
        "c" => {
            let pos = program.table.get_pos();
            for _ in 0..key.count {
                program.table.add_col(pos.col + 1);
                program.table.move_cursor(Direction::Right)
            }
        }
        "C" => {
            let pos = program.table.get_pos();
            for _ in 0..key.count {
                program.table.add_col(pos.col);
            }
        }
        "d" => {
            let mut row = program.table.get_pos().row;
            let count = if key.count >= program.table.get_size()[0] {
                program.table.get_size()[0]
            } else {
                key.count
            };
            for _ in 0..count {
                if row >= program.table.get_size()[0] {
                    row -= 1;
                }
                program.table.remove_row(row);
            }
        }
        "D" => {
            let mut col = program.table.get_pos().col;
            let count = if key.count >= program.table.get_size()[1] {
                program.table.get_size()[1]
            } else {
                key.count
            };
            for _ in 0..count {
                if col >= program.table.get_size()[1] {
                    col -= 1;
                }
                program.table.remove_col(col);
            }
        }
        "w" => {
            let sheet = program.table.to_sheet();
            std::fs::write(program.get_file_path(), sheet).unwrap();
        }
        "x" => {
            let pos = program.table.get_pos();
            program.table.clear_cell(&pos)
        }
        _ => {}
    }
}

fn handle_insert_mode(program: &mut program::Program, key: program::KeySequence) {
    let table = &mut program.table;
    match key.action as u8 {
        //backspace
        127 => table.remove_last_char_in_cell(&table.get_pos()),
        10 => program.set_mode(program::Mode::Normal),
        b'\t' => table.move_cursor(Direction::Right),
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

fn handle_mode(program: &mut program::Program, key: program::KeySequence) {
    match program.current_mode() {
        program::Mode::Normal => handle_normal_mode(program, key),
        program::Mode::Insert => handle_insert_mode(program, key),
        program::Mode::Command => handle_command_mode(program, key),
    }
}

fn setup_terminal() -> termios::Termios {
    let stdin_fd = 0;
    let termios = Termios::from_fd(stdin_fd).unwrap();
    let mut new_termios = termios.clone();
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[termios::VMIN] = 1;
    new_termios.c_cc[termios::VTIME] = 10;
    tcsetattr(stdin_fd, TCSANOW, &mut new_termios).unwrap();
    return termios;
}

fn main() {
    // let mut lexer = calculator::Lexer::new("sum($a1:$b1)/2");
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
    //         &Table::from_sheet_tokens(vec![
    //             sheet_tokenizer::Token::LBracket,
    //             sheet_tokenizer::Token::Number(3.0),
    //             sheet_tokenizer::Token::Number(3.0),
    //             sheet_tokenizer::Token::RBracket,
    //             sheet_tokenizer::Token::LBracket,
    //             sheet_tokenizer::Token::Number(3.0),
    //             sheet_tokenizer::Token::Number(5.0),
    //             sheet_tokenizer::Token::RBracket,
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

    let program_args = parse_args(&mut args);

    let stdin = std::io::stdin();

    let fp = program_args.file;
    let default_ft = "tsheet".to_string();
    let file_type = program_args.opts.get("-f").unwrap_or(&default_ft);

    let mut text = String::new();
    if fp == "-" {
        let mut temp = String::new();
        for line in stdin.lock().lines() {
            temp += &line.unwrap();
        }
        text = temp;
    } else {
        let data = std::fs::read_to_string(&fp);
        if let Ok(t) = data {
            text = t;
        }
    }

    //hack to close pipe on stdin
    let tty = std::fs::File::open("/dev/tty").unwrap();
    let tty_fd = tty.as_raw_fd();
    unsafe {
        libc::dup2(tty_fd, 0);
    }

    let old_termios = setup_terminal();

    //parse data depending on file type
    let mut table: Table = if file_type == "csv" {
        Table::from_csv(&text, ',')
    } else {
        let toks = sheet_tokenizer::parse(text.as_str());
        Table::from_sheet_tokens(toks)
    };

    let mut command_line: CommandLine = CommandLine::new(0, 30);

    let mut program = program::Program::new(&fp, &mut table, &mut command_line);

    let mut reader = stdin;
    while program.running {
        print!("\x1b[2J\x1b[0H");
        let do_equations = match program.current_mode() {
            program::Mode::Insert => false,
            _ => true,
        };
        //TODO: move the actual cursor to the selected row
        println!("{}", program.table.display(10, do_equations));
        println!("{}", program.command_line.display());
        let key_sequence = program.get_key(&mut reader);
        handle_mode(&mut program, key_sequence);
    }
    tcsetattr(0, TCSANOW, &old_termios).unwrap();
}
