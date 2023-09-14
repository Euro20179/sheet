mod sheet_tokenizer;
mod table;

use std::io::{self, Read};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use table::Table;

use crate::table::Direction;

#[derive(Eq, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

fn handle_normal_mode(table: &mut Table, key: u8) {
    match key {
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
        },
        b'r' => {
            let pos = table.get_pos();
            table.add_row(pos.row + 1)
        },
        b'w' => {
            let sheet = table.to_sheet();
            std::fs::write("./test", sheet).unwrap();
        }
        b'x' => {
            let pos = table.get_pos();
            table.clear_cell(&pos)
        }
        _ => {}
    }
}

fn handle_insert_mode(table: &mut Table, key: u8) {
    //TODO: if = is pressed convert the cell to an equation
    table.append_char_to_cell(&table.get_pos(), key as char);
}

fn handle_mode(table: &mut Table, mode: &Mode, key: u8) {
    match mode {
        Mode::Normal => handle_normal_mode(table, key),
        Mode::Insert => handle_insert_mode(table, key),
    }
}

fn main() {
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

    let data = std::fs::read_to_string(fp);
    let mut text = String::from("[]");
    if let Ok(t) = data {
        text = t;
    }
    let toks = sheet_tokenizer::parse(text.as_str());

    let mut table = Table::from_tokens(toks);

    let mut reader = io::stdin();
    let mut mode = Mode::Normal;
    loop {
        print!("\x1b[2J\x1b[0H");
        table.display(10);
        let mut buf = [0; 1];
        reader.read_exact(&mut buf).unwrap();
        let ch = buf[0];
        if ch == b'\'' && mode == Mode::Insert {
            mode = Mode::Normal;
        } else if ch == b'q' && mode == Mode::Normal {
            break;
        } else if ch == b'i' && mode == Mode::Normal {
            mode = Mode::Insert;
        } else if ch == b's' && mode == Mode::Normal {
            table.clear_cell(&table.get_pos());
            mode = Mode::Insert;
        } else {
            handle_mode(&mut table, &mode, ch);
        }
    }

    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}
