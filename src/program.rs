use crate::table::Table;
use crate::command_line::CommandLine;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Command
}

pub struct KeySequence {
    pub count: usize,
    pub action: char,
    pub key: String,
}

pub struct Program<'a> {
    mode: Mode,
    file_path: String,
    pub table: &'a mut Table,
    pub command_line: &'a mut CommandLine,
    pub running: bool
}

impl Program<'_> {
    pub fn new<'a>(fp: &str, table: &'a mut Table, command_line: &'a mut CommandLine) -> Program<'a> {
        Program {
            command_line,
            mode: Mode::Normal,
            table,
            file_path: fp.to_string(),
            running: true
        }
    }

    pub fn current_mode(&self) -> Mode {
        return self.mode;
    }

    pub fn set_mode(&mut self, mode: Mode){
        self.mode = mode;
    }

    pub fn get_file_path(&self) -> &str {
        return self.file_path.as_str();
    }
}
