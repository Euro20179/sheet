use std::{cell::RefCell, collections::HashMap, rc::Rc};

macro_rules! IF {
    ($e:expr, true=>  $t:expr, false=> $f:expr) => {
        if $e {
            $t
        } else {
            $f
        }
    };
}

use crate::{
    calculator::{self, calculate},
    position_parser, sheet_tokenizer,
};

pub fn base_26_to_10(n: String) -> usize {
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
    let mut col_name: String = String::new();
    while n > 0 {
        let modulo = (n - 1) % 26;
        col_name = String::from(('A' as u8 + modulo as u8) as char) + &col_name;
        n = (n - modulo) / 26;
    }
    return col_name;
}

#[derive(Debug, Clone)]
pub enum Data {
    Number(String),
    Equation(String, Option<calculator::CalculatorValue>),
    String(String),
}

fn handle_equation(
    table: &Table,
    expr: &str,
    _invalid_references: &mut Vec<(usize, usize)>,
) -> Result<String, &'static str> {
    let mut map: HashMap<String, calculator::CalculatorValue> = HashMap::new();

    map.insert("%recursion".to_string(), calculator::CalculatorValue::Number(0.0));

    let ans = match calculate(expr, &mut map, table) {
        Ok(calculator::CalculatorValue::String(s)) => s.to_string(),
        Ok(calculator::CalculatorValue::Number(n)) => n.to_string(),
        Ok(calculator::CalculatorValue::Range(x, y)) => format!("{:?}..{:?}", x, y),
        Err(e) => {
            match e {
                calculator::CalculatorError::RecursionLimit => "Err#1: Recursion limit reached",
                calculator::CalculatorError::InvalidBinaryOp(_) => "Err#2: Invalid binary operation",
            }.to_owned()
        }
    };
    return Ok(ans);
}

impl Data {
    fn display_number(&self, n: &str, max_width: usize, is_hovered: bool) -> String {
        let new_text = n.to_owned();
        return IF!(n.len() > max_width && !is_hovered,
            true=> new_text[0..max_width].to_string(),
            false=> format!("{:<max_width$}", new_text, max_width = max_width)
        );
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
    //FIXME: bug where if there is not a trailing comma in each row, the row is empty
    if array.len() < 1 {
        return 0;
    }
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

    pub fn get_size(&self) -> [usize; 2] {
        return [self.rows.len(), self.columns.len()];
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
        self.pad_col(&mut col);
        self.columns.insert(col_no, col);
        self.column_sizes.insert(col_no, 10);
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

    pub fn set_cursor_pos(&mut self, row_no: usize, col_no: usize) {
        if row_no < self.rows.len() {
            self.current_pos.row = row_no;
        }
        if col_no < self.columns.len() {
            self.current_pos.col = col_no;
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
        if self.rows.len() == 0 {
            return true;
        }
        return self.get_pos().row == self.rows.len() - 1;
    }

    pub fn cursor_at_right(&self) -> bool {
        if self.columns.len() == 0 {
            return true;
        }
        return self.current_pos.col == self.columns.len() - 1;
    }

    pub fn remove_last_char_in_cell(&mut self, position: &Position) {
        let data = self.get_value_at_position(position);
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

    pub fn append_text_to_cell(&mut self, position: &Position, char: String) {
        let data = self.get_value_at_position(position);
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
        match &self.columns[self.current_pos.col][self.current_pos.row] {
            Data::String(s) | Data::Equation(s, ..) => {
                if s == "" {
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }

    pub fn convert_cell(&mut self, pos: &Position, t: Data) {
        self.rows[pos.row][pos.col] = t.clone();
        self.columns[pos.col][pos.row] = t;
    }

    fn find_displayable_rows(&self, rows_to_view: usize) -> [usize; 2] {
        let mut rows_above = rows_to_view / 2;
        let mut rows_below = rows_to_view / 2;
        if rows_above > self.current_pos.row {
            rows_above = self.current_pos.row;
        }
        if self.current_pos.row + rows_below >= self.rows.len() {
            rows_below = self.rows.len() - self.current_pos.row;
        }
        return [
            self.current_pos.row - rows_above,
            self.current_pos.row + rows_below,
        ];
    }

    fn find_displayable_cols(&self, cols_to_view: usize) -> [usize; 2] {
        let mut cols_left = cols_to_view / 2;
        let mut cols_right = cols_to_view / 2;
        if cols_left > self.current_pos.col {
            cols_left = self.current_pos.col;
        }
        if self.current_pos.col + cols_right >= self.columns.len() {
            cols_right = self.columns.len() - self.current_pos.col;
        }
        return [
            self.current_pos.col - cols_left,
            self.current_pos.col + cols_right,
        ];
    }

    pub fn display(&self, max_width: usize, do_equations: bool) -> String {
        let mut text = format!("{:<max_width$}", " ", max_width = max_width);
        let row_slice = self.find_displayable_rows(30); //TODO: make this not hardcoded
        let col_slice = self.find_displayable_cols(6); //TODO: make this not hardcoded
        let mut row_no = row_slice[0];
        for i in col_slice[0]..col_slice[1] {
            text += &format!(
                "{:^max_width$}",
                base_10_to_col_num(i + 1),
                max_width = self.column_sizes[i]
            );
        }
        text += &String::from("\n");
        for row in &self.rows[row_slice[0]..row_slice[1]] {
            let mut col_no = col_slice[0];
            text += &format!(
                "{:^max_width$}",
                &(row_no + 1).to_string(),
                max_width = max_width
            );
            for item in &row[col_slice[0]..col_slice[1]] {
                let is_selected = self.is_current_pos(row_no, col_no);
                let display_text =
                    item.display(self, self.column_sizes[col_no], do_equations, is_selected);
                text += &(IF!(is_selected,
                    true => format!("\x1b[7m{}\x1b[0m", display_text),
                    false => display_text
                ));
                col_no += 1;
            }
            text += &"\n".to_owned();
            row_no += 1;
        }
        return text;
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
                cur_col.push(IF!(i >= row.len(),
                    true => Data::Number("0".to_string()),
                    false => row[i].clone()
                ));
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

    pub fn from_csv(text: &str, seperator: char) -> Table {
        let mut rows: Vec<Vec<Data>> = vec![];
        let mut cur_row: Vec<Data> = vec![];
        let mut cur_item: String = String::new();
        for ch in text.chars() {
            if ch == seperator {
                if let Ok(n) = cur_item.parse::<f64>() {
                    cur_row.push(Data::Number(n.to_string()))
                } else {
                    cur_row.push(Data::String(cur_item));
                }
                cur_item = String::new();
            } else if ch == '\n' {
                if let Ok(n) = cur_item.parse::<f64>() {
                    cur_row.push(Data::Number(n.to_string()))
                } else {
                    cur_row.push(Data::String(cur_item));
                }
                rows.push(cur_row);
                cur_row = vec![];
                cur_item = String::new();
            } else {
                cur_item += &String::from(ch);
            }
        }
        if let Ok(n) = cur_item.parse::<f64>() {
            cur_row.push(Data::Number(n.to_string()))
        } else {
            cur_row.push(Data::String(cur_item));
        }
        if rows.len() > 0 {
            if cur_row.len() == rows[0].len() + 1 {
                //only append if it would make all rows equal size
                rows.push(cur_row)
            }
        }
        let mut column_sizes: Vec<usize> = vec![];
        for _ in 0..rows.len() {
            column_sizes.push(10);
        }
        let columns = Table::build_columns_from_rows(&rows);
        Table {
            rows,
            column_sizes,
            columns,
            current_pos: Position { row: 0, col: 0 },
        }
    }

    pub fn from_sheet_tokens(toks: Vec<sheet_tokenizer::Token>) -> Table {
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

        let column_sizes: Vec<usize>;
        if rows.len() > 0 {
            column_sizes = rows
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
        } else {
            rows.push(vec![Data::String(String::from(""))]);
            column_sizes = vec![10];
        }
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
