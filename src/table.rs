use crate::sheet_tokenizer;

#[derive(Debug, Clone)]
pub enum Data {
    Number(f64),
    Equation(String),
    String(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct Table {
    rows: Vec<Vec<Data>>,
    columns: Vec<Vec<Data>>,
    current_pos: Position,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Bottom,
    Top,
    MostLeft,
    MostRight,
}

fn largest_list_in_2d_array<T>(array: &Vec<Vec<T>>) -> usize {
    let mut largest = array[0].len();
    for arr in array {
        if arr.len() > largest {
            largest = arr.len();
        }
    }
    return largest;
}

impl Table {
    pub fn get_pos(&self) -> Position {
        return self.current_pos;
    }

    pub fn get_data_at_pos(&self, pos: &Position) -> &Data {
        return &self.columns[pos.col][pos.row];
    }

    pub fn add_row(&mut self, row_no: usize) {
        let mut row: Vec<Data> = vec![];
        self.pad_row(&mut row);
        self.rows.insert(row_no, row);
        for column in &mut self.columns {
            column.insert(row_no, Data::Number(0.0));
        }
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.current_pos.row > 0 {
                    self.current_pos.row -= 1;
                }
            }
            Direction::Down => {
                self.current_pos.row += 1;
                if self.current_pos.row >= self.rows.len() {
                    self.current_pos.row = self.rows.len() - 1;
                }
            }
            Direction::Left => {
                if self.current_pos.col > 0 {
                    self.current_pos.col -= 1;
                }
            }
            Direction::Right => {
                self.current_pos.col += 1;
                if self.current_pos.col >= self.columns.len() {
                    self.current_pos.col = self.columns.len() - 1
                }
            }
            Direction::Bottom => {
                self.current_pos.row = self.rows.len() - 1;
            }
            Direction::Top => self.current_pos.row = 0,
            Direction::MostLeft => {
                self.current_pos.col = 0;
            }
            Direction::MostRight => {
                self.current_pos.col = self.columns.len() - 1;
            }
        }
    }

    pub fn set_value_at_position(&mut self, position: &Position, value: Data) {
        self.columns[position.col][position.row] = value.clone();
        self.rows[position.row][position.col] = value;
    }

    pub fn clear_cell(&mut self, position: &Position) {
        self.set_value_at_position(position, Data::String("".to_string()));
    }

    pub fn append_char_to_cell(&mut self, position: &Position, char: char) {
        let data = self.get_data_at_pos(position);
        match data {
            Data::Number(n) => {
                let char_digit: Result<f64, _> = char.to_string().parse();
                if let Ok(digit) = char_digit {
                    let mut new_n = n.to_owned();
                    new_n *= 10.0;
                    new_n += digit;
                    self.set_value_at_position(position, Data::Number(new_n));
                } else {
                    let mut str = n.to_string();
                    str += &char.to_string();
                    self.set_value_at_position(position, Data::String(str))
                }
            }
            Data::Equation(s) => {
                let mut new_str = s.to_owned();
                new_str += &char.to_string();
                self.set_value_at_position(position, Data::Equation(new_str))
            }
            Data::String(s) => {
                let mut new_str = s.to_owned();
                new_str += &char.to_string();
                self.set_value_at_position(position, Data::String(new_str))
            }
        }
    }

    fn is_current_pos(&self, row_no: usize, col_no: usize) -> bool {
        return row_no == self.current_pos.row && col_no == self.current_pos.col;
    }

    pub fn display(&self, max_width: usize) {
        let mut text = String::new();
        let mut row_no = 0;
        for row in &self.rows {
            let mut col_no = 0;
            for item in row {
                match item {
                    Data::String(s) | Data::Equation(s) => {
                        if self.is_current_pos(row_no, col_no) {
                            text += &String::from("\x1b[41m")
                        }
                        let new_text =
                            format!("{:<max_width$}", s, max_width = max_width).to_owned();
                        if new_text.len() > max_width && !self.is_current_pos(row_no, col_no) {
                            text += &new_text[0..max_width];
                        } else {
                            text += &new_text;
                        }
                        if self.is_current_pos(row_no, col_no) {
                            text += &String::from("\x1b[0m")
                        }
                    }
                    Data::Number(n) => {
                        if self.is_current_pos(row_no, col_no) {
                            text += &String::from("\x1b[41m")
                        }
                        let new_text =
                            format!("{:<max_width$}", n, max_width = max_width).to_owned();
                        if new_text.len() > max_width && !self.is_current_pos(row_no, col_no) {
                            text += &new_text[0..max_width];
                        } else {
                            text += &new_text;
                        }
                        if self.is_current_pos(row_no, col_no) {
                            text += &String::from("\x1b[0m")
                        }
                    }
                }
                col_no += 1;
            }
            text += &"\n".to_owned();
            row_no += 1;
        }
        println!("{}", text);
    }

    pub fn to_sheet(&self) -> String {
        let mut text = String::new();

        for row in &self.rows {
            text += &String::from("[");
            for item in row {
                match item {
                    Data::String(t) => {
                        text += &format!("\"{}\"", t).to_string();
                    },
                    Data::Number(n) => {
                        text += &format!("{}", n).to_string();
                    },
                    Data::Equation(t) => {
                        text += &format!("({})", t).to_string();
                    }
                }
                text += &String::from(",");
            }
            text += &String::from("]");
            text += &String::from("\n");
        }
        return text;
    }

    fn build_columns_from_rows(rows: &Vec<Vec<Data>>) -> Vec<Vec<Data>> {
        let mut columns: Vec<Vec<Data>> = vec![];
        let largest_row = largest_list_in_2d_array(rows);

        for i in 0..largest_row {
            let mut cur_col: Vec<Data> = vec![];
            for row in rows {
                if i >= row.len() {
                    cur_col.push(Data::Number(0.0));
                } else {
                    cur_col.push(row[i].clone());
                }
            }
            columns.push(cur_col);
        }
        return columns;
    }

    fn pad_row(&self, row: &mut Vec<Data>){
        let largest_row = largest_list_in_2d_array(&self.rows);

        if row.len() < largest_row {
            for _ in row.len()..largest_row {
                row.push(Data::Number(0.0));
            }
        }
    }

    fn pad_rows(rows: &mut Vec<Vec<Data>>) {
        let largest_row = largest_list_in_2d_array(rows);

        for row in rows {
            if row.len() < largest_row {
                for _ in row.len()..largest_row {
                    row.push(Data::Number(0.0));
                }
            }
        }
    }

    pub fn from_tokens(toks: Vec<sheet_tokenizer::Token>) -> Table {
        let mut rows: Vec<Vec<Data>> = vec![];
        let mut iter_toks = toks.into_iter();
        type T = sheet_tokenizer::Token;
        while let Some(T::LBracket) = iter_toks.next() {
            let mut current_row: Vec<Data> = vec![];
            loop {
                let tok = iter_toks.next();
                match tok {
                    Some(T::RBracket) => {
                        rows.push(current_row);
                        break;
                    }
                    Some(T::String(text)) => {
                        current_row.push(Data::String(text));
                    }
                    Some(T::Expr(text)) => {
                        current_row.push(Data::Equation(text));
                    }
                    Some(T::Number(n)) => {
                        current_row.push(Data::Number(n));
                    }
                    None => break,
                    _ => continue,
                }
            }
        }

        let columns = Table::build_columns_from_rows(&rows);
        Table::pad_rows(&mut rows);
        return Table {
            rows,
            columns,
            current_pos: Position { row: 0, col: 0 },
        };
    }
}
