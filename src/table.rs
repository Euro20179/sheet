use std::collections::HashMap;

use crate::{
    calculator::{self, calculate},
    position_parser, sheet_tokenizer,
};

fn base_26_to_10(n: String) -> usize {
    let mut ans = 0;
    let reversed = n.chars().rev().collect::<String>();
    let base: usize = 26;
    for i in 0..reversed.len() {
        let ch = reversed.chars().nth(i).unwrap() as u8;
        ans += ((ch - 97) as usize) * (base.pow(i as u32));
    }
    return ans;
}

pub fn base_10_to_col_num(mut n: usize) -> String {
    if n < 26 {
        return String::from((n + 65) as u8 as char);
    }

    let mut ans = String::new();
    let mut place = 0;
    while n >= 26 {
        if place > 0 {
            ans += &String::from(((n % 26) + 64) as u8 as char);
        } else {
            ans += &String::from(((n % 26) + 65) as u8 as char);
        }
        n /= 26;
        place += 1;
    }
    if place > 0 {
        ans += &String::from(((n % 26) + 64) as u8 as char);
    } else {
        ans += &String::from(((n % 26) + 65) as u8 as char);
    }
    return ans.chars().rev().collect();
}

#[derive(Debug, Clone)]
pub enum Data {
    Number(String),
    Equation(String, Option<calculator::Result>),
    String(String),
}

fn handle_equation(
    table: &Table,
    expr: &str,
    _invalid_references: &mut Vec<(usize, usize)>,
) -> Result<String, &'static str> {
    let map: HashMap<String, calculator::Result> = HashMap::new();

    let ans = match calculate(expr.to_owned(), &map, table) {
        calculator::Result::String(s) => s.to_string(),
        calculator::Result::Number(n) => n.to_string(),
        calculator::Result::Range(x, y) => format!("{:?}..{:?}", x, y),
    };
    return Ok(ans);
}

impl Data {
    fn display_number(&self, n: &str, max_width: usize, is_hovered: bool) -> String {
        let new_text = n.to_owned();
        if n.len() > max_width && !is_hovered {
            return new_text[0..max_width].to_string();
        } else {
            return format!("{:<max_width$}", new_text, max_width = max_width);
        }
    }

    fn display_equation(
        &self,
        table: &Table,
        e: &str,
        max_width: usize,
        do_equations: bool,
        is_hovered: bool,
    ) -> String {
        if !do_equations {
            return format!("{:<max_width$}", e, max_width = max_width);
        }
        let mut invalid_refs: Vec<(usize, usize)> = vec![];
        let ans = handle_equation(table, e, &mut invalid_refs);
        if let Ok(a) = ans {
            if a.len() > max_width && !is_hovered {
                return a[0..max_width].to_string();
            }
            return format!("{:<max_width$}", a, max_width = max_width);
        } else {
            return format!("{:<max_width$}", "Inf ref", max_width = max_width);
        }
    }

    fn display_string(&self, s: &str, max_width: usize, is_hovered: bool) -> String {
        let new_text = s.to_owned();
        if new_text.len() > max_width && !is_hovered {
            return new_text[0..max_width].to_string();
        } else {
            return format!("{:<max_width$}", new_text, max_width = max_width);
        }
    }

    pub fn display(
        &self,
        table: &Table,
        max_width: usize,
        do_equations: bool,
        is_hovered: bool,
    ) -> String {
        match self {
            Data::Number(n) => self.display_number(n, max_width, is_hovered),
            Data::String(s) => self.display_string(s, max_width, is_hovered),
            Data::Equation(e, ..) => {
                self.display_equation(table, e, max_width, do_equations, is_hovered)
            }
        }
    }
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
    column_sizes: Vec<usize>,
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
            column.insert(row_no, Data::String("".to_string()));
        }
    }

    pub fn add_col(&mut self, col_no: usize) {
        let mut col: Vec<Data> = vec![];
        self.column_sizes.insert(col_no, 10);
        self.pad_col(&mut col);
        self.columns.insert(col_no, col);
        for row in &mut self.rows {
            row.insert(col_no, Data::String("".to_string()));
        }
    }

    pub fn remove_col(&mut self, col_no: usize) {
        self.column_sizes.remove(col_no);
        self.columns.remove(col_no);
        for row in &mut self.rows {
            row.remove(col_no);
        }
        if col_no >= self.columns.len() {
            self.move_cursor(Direction::Left);
        }
    }

    pub fn remove_row(&mut self, row_no: usize) {
        self.rows.remove(row_no);
        for col in &mut self.columns {
            col.remove(row_no);
        }
        if row_no >= self.rows.len() {
            self.move_cursor(Direction::Up);
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

    pub fn get_value_at_position(&self, position: &Position) -> Data {
        if position.row >= self.rows.len() {
            return Data::String("0".to_string());
        }
        let row = &self.rows[position.row];
        if position.col >= row.len() {
            return Data::String("0".to_string());
        }
        return row[position.col].clone();
    }

    pub fn get_values_at_range(&self, start: &Position, end: &Position) -> Vec<Data> {
        let mut items: Vec<Data> = vec![];
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                items.push(self.get_value_at_position(&Position { row, col }))
            }
        }
        items
    }

    pub fn clear_cell(&mut self, position: &Position) {
        self.set_value_at_position(position, Data::String("".to_string()));
    }

    pub fn cursor_at_bottom(&self) -> bool {
        return self.get_pos().row == self.rows.len() - 1;
    }

    pub fn remove_last_char_in_cell(&mut self, position: &Position) {
        let data = self.get_data_at_pos(position);
        let new_value = match data {
            Data::Equation(s, c) => {
                let mut new_str = s.to_owned();
                if new_str.len() == 0 {
                    Data::Equation(new_str, c.to_owned())
                } else {
                    new_str = new_str[0..new_str.len() - 1].to_string();
                    Data::Equation(new_str, c.to_owned())
                }
            }
            Data::String(s) => {
                let mut new_str = s.to_owned();
                if new_str.len() == 0 {
                    Data::String("".to_string())
                } else {
                    new_str = new_str[0..new_str.len() - 1].to_string();
                    Data::String(new_str)
                }
            }
            Data::Number(s) => {
                let mut new_str = s.to_owned();
                if new_str.len() == 0 {
                    Data::String("".to_string())
                } else {
                    new_str = new_str[0..new_str.len() - 1].to_string();
                    Data::Number(new_str)
                }
            }
        };
        self.set_value_at_position(position, new_value);
    }

    pub fn append_char_to_cell(&mut self, position: &Position, char: char) {
        let data = self.get_data_at_pos(position);
        match data {
            Data::Number(n) => {
                let mut new_str = n.to_string();
                new_str += &String::from(char);
                if let Ok(_) = new_str.parse::<f64>() {
                    self.set_value_at_position(position, Data::Number(new_str));
                } else {
                    self.set_value_at_position(position, Data::String(new_str));
                }
            }
            Data::Equation(s, c) => {
                let mut new_str = s.to_owned();
                new_str += &char.to_string();
                self.set_value_at_position(position, Data::Equation(new_str, c.to_owned()))
            }
            Data::String(s) => {
                let mut new_str = s.to_owned();
                new_str += &char.to_string();
                if let Ok(_) = new_str.parse::<f64>() {
                    self.set_value_at_position(position, Data::Number(new_str));
                } else {
                    self.set_value_at_position(position, Data::String(new_str))
                }
            }
        }
    }

    fn is_current_pos(&self, row_no: usize, col_no: usize) -> bool {
        return row_no == self.current_pos.row && col_no == self.current_pos.col;
    }

    pub fn human_position_to_position(&self, position: String) -> Position {
        let toks = position_parser::Lexer::new(position).lex();

        let mut col_pos = 0;
        let mut row_pos = 0;

        for tok in toks {
            match tok {
                position_parser::Token::Col(c) => {
                    col_pos = base_26_to_10(c);
                }
                position_parser::Token::Row(r) => {
                    row_pos = r - 1;
                }
            }
        }

        return Position {
            row: row_pos,
            col: col_pos,
        };
    }

    pub fn cursor_pos_is_empty(&self) -> bool {
        if let Data::String(s) = &self.columns[self.current_pos.col][self.current_pos.row] {
            if s == "" {
                return true;
            }
        }
        return false;
    }

    pub fn convert_cell(&mut self, pos: &Position, t: Data) {
        self.columns[pos.col][pos.row] = match t {
            Data::String(..) => Data::String(String::from("")),
            Data::Number(..) => Data::Number(String::from("0")),
            Data::Equation(..) => Data::Equation(String::from(""), None),
        }
    }

    pub fn display(&self, max_width: usize, do_equations: bool) {
        let mut text = format!("{:<max_width$}", " ", max_width = max_width);
        let mut row_no = self.current_pos.row;
        let row_view = 20;
        let col_view = 5;
        let end_row = if self.current_pos.row + row_view > self.rows.len() {
            self.rows.len()
        } else {
            self.current_pos.row + row_view
        };
        let end_col = if self.current_pos.col + col_view > self.columns.len() {
            self.columns.len()
        } else {
            self.current_pos.col + col_view
        };
        for i in self.current_pos.col..end_col {
            text += &format!(
                "{:^max_width$}",
                base_10_to_col_num(i),
                max_width = self.column_sizes[i]
            );
        }
        text += &String::from("\n");
        for row in &self.rows[self.current_pos.row..end_row] {
            let mut col_no = self.current_pos.col;
            text += &format!(
                "{:^max_width$}",
                &(row_no + 1).to_string(),
                max_width = max_width
            );
            for item in &row[self.current_pos.col..end_col] {
                if self.is_current_pos(row_no, col_no) {
                    text += &String::from("\x1b[41m");
                    text += &item.display(self, self.column_sizes[col_no], do_equations, true);
                    text += &String::from("\x1b[0m")
                } else {
                    text += &item.display(self, self.column_sizes[col_no], do_equations, false);
                }
                col_no += 1;
            }
            text += &"\n".to_owned();
            row_no += 1;
        }
        println!("{}", text);
    }

    pub fn to_sheet(&self) -> String {
        let mut text = "[".to_string();

        for size in &self.column_sizes {
            text += &(size.to_string() + &String::from(","));
        }
        text += &"]\n".to_string();

        for row in &self.rows {
            text += &String::from("[");
            for item in row {
                match item {
                    Data::String(t) => {
                        text += &format!("\"{}\"", t).to_string();
                    }
                    Data::Number(n) => {
                        if let Ok(n) = n.parse::<f64>() {
                            text += &format!("{}", n as f64).to_string();
                        } else {
                            panic!("{:?} Is not a float", n)
                        }
                    }
                    Data::Equation(t, ..) => {
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
                    cur_col.push(Data::Number("0".to_string()));
                } else {
                    cur_col.push(row[i].clone());
                }
            }
            columns.push(cur_col);
        }
        return columns;
    }

    fn pad_col(&self, col: &mut Vec<Data>) {
        let largest_col = largest_list_in_2d_array(&self.columns);

        if col.len() < largest_col {
            for _ in col.len()..largest_col {
                col.push(Data::String("".to_string()));
            }
        }
    }

    fn pad_row(&self, row: &mut Vec<Data>) {
        let largest_row = largest_list_in_2d_array(&self.rows);

        if row.len() < largest_row {
            for _ in row.len()..largest_row {
                row.push(Data::String("".to_string()));
            }
        }
    }

    fn pad_rows(rows: &mut Vec<Vec<Data>>) {
        let largest_row = largest_list_in_2d_array(rows);

        for row in rows {
            if row.len() < largest_row {
                for _ in row.len()..largest_row {
                    row.push(Data::Number("0".to_string()));
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
                        current_row.push(Data::Equation(text, None));
                    }
                    Some(T::Number(n)) => {
                        current_row.push(Data::Number(n.to_string()));
                    }
                    None => break,
                    _ => continue,
                }
            }
        }

        let column_sizes: Vec<usize> = rows
            .remove(0)
            .into_iter()
            .map(|d| {
                if let Data::Number(n) = d {
                    let size: usize = n.parse().unwrap();
                    return size;
                }
                return 10;
            })
            .collect();

        let columns = Table::build_columns_from_rows(&rows);
        Table::pad_rows(&mut rows);
        return Table {
            rows,
            columns,
            current_pos: Position { row: 0, col: 0 },
            column_sizes,
        };
    }
}
