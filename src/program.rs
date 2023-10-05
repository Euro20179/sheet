use std::io::{Read, Stdin};
use std::rc::Rc;

use crate::command_line::CommandLine;
use crate::table::Table;
use crate::undo_tree::{self, UndoTree};

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

pub struct TermInfo {
    pub cols: usize,
    pub lines: usize,
}

pub struct Program<'a> {
    mode: Mode,
    file_path: String,
    undo_tree: undo_tree::UndoTree,
    pub table: &'a mut Table,
    pub command_line: &'a mut CommandLine,
    pub running: bool,
    pub term_info: TermInfo,
    pub previous_tables: Vec<Table>,
}

impl Program<'_> {
    pub fn new<'a>(
        fp: &str,
        table: &'a mut Table,
        command_line: &'a mut CommandLine,
    ) -> Program<'a> {
        let rows = table.get_rows();
        Program {
            command_line,
            mode: Mode::Normal,
            table,
            file_path: fp.to_string(),
            running: true,
            term_info: TermInfo {
                cols: 30,
                lines: 20,
            },
            previous_tables: vec![],
            undo_tree: UndoTree::new(Box::new(rows)),
        }
    }

    pub fn save_state(&mut self) {
        self.undo_tree = self.undo_tree.save(Box::new(self.table.get_rows()));
    }

    pub fn undo(&mut self) {
        let tree = self.undo_tree.undo();
        match tree {
            None => self.command_line.print("Cannot undo"),
            Some(t) => {
                self.command_line.print("Undo");
                self.table.set_data(*t.get_state());
                self.undo_tree = *t;
            }
        }
    }

    pub fn redo(&mut self) {
        let tree = self.undo_tree.redo();
        match tree {
            None => self.command_line.print("Cannot redo"),
            Some(t) => {
                self.command_line.print("Redo");
                self.table.set_data(*t.get_state());
                self.undo_tree = *t;
            }
        }
    }

    pub fn current_mode(&self) -> Mode {
        return self.mode;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn is_mode(&self, mode: Mode) -> bool {
        if self.mode == mode {
            return true;
        }
        return false;
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
                    count: 1,
                };
            }
        }
    }
}
