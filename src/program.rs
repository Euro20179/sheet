use std::io::{Read, Stdin};

use crate::command_line::CommandLine;
use crate::table::Table;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct KeySequence {
    pub count: usize,
    pub action: char,
    pub key: String,
}

pub struct TermInfo{
    pub cols: usize,
    pub lines: usize
}

pub struct Program<'a> {
    mode: Mode,
    file_path: String,
    pub table: &'a mut Table,
    pub command_line: &'a mut CommandLine,
    pub running: bool,
    pub term_info: TermInfo
}

impl Program<'_> {
    pub fn new<'a>(
        fp: &str,
        table: &'a mut Table,
        command_line: &'a mut CommandLine,
    ) -> Program<'a> {
        Program {
            command_line,
            mode: Mode::Normal,
            table,
            file_path: fp.to_string(),
            running: true,
            term_info: TermInfo { cols: 30, lines: 20 }
        }
    }

    pub fn current_mode(&self) -> Mode {
        return self.mode;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn get_file_path(&self) -> &str {
        return self.file_path.as_str();
    }

    pub fn get_key(&self, reader: &mut Stdin) -> KeySequence {
        let mut count = String::new();
        let mut buf = [0; 32]; //consume enough bytes to store utf-8, 32 bytes should be enough
        match self.mode {
            Mode::Normal => loop {
                let bytes_read = reader.read(&mut buf).unwrap();
                let key = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();
                let ch = buf[0];

                if ch >= 48 && ch <= 57 {
                    count += &String::from(ch as char);
                } else {
                    if count == "" {
                        count = String::from("1");
                    }
                    return KeySequence {
                        action: ch as char,
                        count: count.parse().unwrap(),
                        key,
                    };
                }
            },
            Mode::Insert | Mode::Command => {
                let bytes_read = reader.read(&mut buf).unwrap();
                let key = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();
                let ch = buf[0];
                return KeySequence {
                    action: ch as char,
                    key,
                    count: 1
                };
            },
        }
    }
}
