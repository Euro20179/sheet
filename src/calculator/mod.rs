use std::{collections::HashMap, str::Chars};

use crate::table::{self, Data, Position, Table};

static RECURSION_LIMIT: f64 = 1000.0;

#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Star,
    Div,
    Number(f64),
    LParen,
    RParen,
    Ident(String),
    Colon,
    Comma,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    cur_char: Option<char>,
    chars: Chars<'a>,
}

impl Lexer<'_> {
    pub fn new<'a>(content: &'a str) -> Lexer<'a> {
        let chars = content.chars();
        Lexer {
            cur_char: None,
            chars,
        }
    }

    fn next(&mut self) -> Option<char> {
        self.cur_char = self.chars.next();
        return self.cur_char;
    }

    fn build_number(&mut self) -> f64 {
        let mut text = self.cur_char.unwrap().to_string();
        let mut is_dec = false;
        while let Some(ch) = self.next() {
            let char = ch;
            if char == '.' && !is_dec {
                is_dec = true;
            } else if char == '.' {
                break;
            } else if match char {
                '0'..='9' => false,
                _ => true,
            } {
                break;
            }
            text += &char.to_string();
        }

        let n: f64 = text.parse().expect("Could not parse number");
        return n;
    }

    fn build_ident(&mut self) -> String {
        let mut ident = self.cur_char.unwrap().to_string();
        while let Some(ch) = self.next() {
            match ch {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                    ident += &String::from(ch);
                }
                _ => break,
            }
        }
        return ident;
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        self.next();
        loop {
            if let None = self.cur_char {
                break;
            } else if let Some(ch) = self.cur_char {
                let tok = match ch {
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Star,
                    '/' => Token::Div,
                    ')' => Token::RParen,
                    '(' => Token::LParen,
                    '$' => {
                        let tok = Token::Ident(self.build_ident());
                        tokens.push(tok);
                        continue;
                    }
                    ':' => Token::Colon,
                    ',' => Token::Comma,
                    '0'..='9' => {
                        let tok = Token::Number(self.build_number());
                        tokens.push(tok);
                        continue;
                    }
                    ' ' | '\t' | '\n' => {
                        self.next();
                        continue;
                    }
                    'A'..='Z' | 'a'..='z' | '_' => {
                        let tok = Token::Ident(self.build_ident());
                        tokens.push(tok);
                        continue;
                    }
                    _ => Token::Number(0.0),
                };
                tokens.push(tok);
            }
            self.next();
        }
        return tokens;
    }
}

#[derive(Debug)]
pub enum Operation {
    Mul,
    Div,
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum Node {
    BinOp(Box<Node>, Operation, Box<Node>),
    Number(f64),
    Ident(String),
    Range(String, String),
    Call(String, Vec<Node>),
}

impl Node {
    pub fn visit(
        &self,
        symbols: &mut HashMap<String, CalculatorValue>,
        table: &Table,
    ) -> Result<CalculatorValue, u8> {
        match self {
            Node::Call(fn_name, nodes) => {
                let mut values: Vec<CalculatorValue> = vec![];
                //not using map because closures require a special borrow that breaks everything
                for node in nodes {
                    if let Ok(v) = node.visit(symbols, table) {
                        values.push(v);
                    }
                }
                if fn_name == "rand" {
                    return Ok(CalculatorValue::Number(rand::random()));
                } else if fn_name == "sum" {
                    let mut sum = 0.0;
                    for value in values {
                        sum += value.to_f64(symbols, table);
                    }
                    return Ok(CalculatorValue::Number(sum));
                } else if fn_name == "mean" {
                    let mut sum = 0.0;
                    let mut real_count = 0; //dont count strings
                    for value in values {
                        match value {
                            CalculatorValue::Number(n) => {
                                sum += n;
                                real_count += 1;
                            }
                            CalculatorValue::Range(start, end) => {
                                let vals = table.get_values_at_range(&start, &end);
                                for val in vals {
                                    match val {
                                        table::Data::Number(n) => {
                                            sum += n.parse().unwrap_or(0.0);
                                            real_count += 1;
                                        }
                                        table::Data::Equation(e, cache) => {
                                            if let Some(r) = cache {
                                                sum += r.to_f64(symbols, table);
                                            } else {
                                                sum += calculate(&e, symbols, table)?
                                                    .to_f64(symbols, table);
                                            }
                                            real_count += 1;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    return Ok(CalculatorValue::Number(sum / real_count as f64));
                }
                return Ok(CalculatorValue::Number(0.0));
            }
            Node::Ident(s) => {
                if s.chars().nth(0).unwrap_or('a') != '$' {
                    Ok(symbols
                        .get(s)
                        .unwrap_or(&CalculatorValue::Number(0.0))
                        .to_owned())
                } else {
                    let name = s[1..].to_string();
                    let pos = table.human_position_to_position(name);
                    let val = table.get_value_at_position(&pos);
                    let res_value = match val {
                        Data::Number(n) => {
                            let num: f64 = n.parse().unwrap();
                            return Ok(CalculatorValue::Number(num));
                        }
                        Data::String(a) => CalculatorValue::String(a),
                        Data::Equation(e, _cache) => {
                            //FIXME: can be infinitely recursive when self referencing occurs
                            return calculate(&e, symbols, table);
                        }
                    };
                    return Ok(res_value);
                }
            }
            Node::Range(start, finish) => {
                let start_pos = table.human_position_to_position(start[1..].to_owned());
                let end_pos = table.human_position_to_position(finish[1..].to_owned());
                Ok(CalculatorValue::Range(start_pos, end_pos))
            }
            Node::Number(n) => Ok(CalculatorValue::Number(n.to_owned())),
            Node::BinOp(left, op, right) => {
                let left_val = left.visit(symbols, table);
                let right_val = right.visit(symbols, table);

                match left_val {
                    Ok(CalculatorValue::Number(n)) => match right_val {
                        Ok(CalculatorValue::Number(n2)) => match op {
                            Operation::Mul => Ok(CalculatorValue::Number(n * n2)),
                            Operation::Div => Ok(CalculatorValue::Number(n / n2)),
                            Operation::Plus => Ok(CalculatorValue::Number(n + n2)),
                            Operation::Minus => Ok(CalculatorValue::Number(n - n2)),
                        },
                        _ => Ok(CalculatorValue::Number(0.0)),
                    },
                    _ => Ok(CalculatorValue::Number(0.0)),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    cur_pos: usize,
    tokens: Vec<Token>,
    cur_tok: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            cur_pos: 0,
            cur_tok: tokens[0].clone(),
            tokens,
        }
    }

    fn next(&mut self) -> bool {
        self.cur_pos += 1;
        if self.cur_pos >= self.tokens.len() {
            return false;
        }
        self.cur_tok = self.tokens[self.cur_pos].clone();
        return true;
    }

    fn factor(&mut self) -> Node {
        match self.cur_tok.clone() {
            Token::Number(n) => {
                self.next();
                return Node::Number(n);
            }
            Token::Ident(i) => {
                self.next();
                if let Token::Colon = self.cur_tok {
                    self.next();
                    if let Token::Ident(i2) = self.cur_tok.clone() {
                        self.next();
                        return Node::Range(i.to_string(), i2.to_string());
                    }
                } else if let Token::LParen = self.cur_tok {
                    self.next();
                    let mut nodes: Vec<Node> = vec![];
                    while match self.cur_tok {
                        Token::RParen => false,
                        _ => true,
                    } {
                        nodes.push(self.expr());
                        if let Token::Comma = self.cur_tok {
                            self.next();
                        } else {
                            break;
                        }
                    }
                    self.next();
                    return Node::Call(i.to_string(), nodes);
                }
                return Node::Ident(i.to_string());
            }
            Token::LParen => {
                self.next();
                let node = self.expr();
                self.next();
                return node;
            }
            _ => Node::Number(0.0),
        }
    }

    fn term(&mut self) -> Node {
        let mut left = self.factor();
        loop {
            match self.cur_tok {
                Token::Div | Token::Star => {}
                _ => break,
            }
            let op = match self.cur_tok {
                Token::Star => Operation::Mul,
                Token::Div => Operation::Div,
                _ => todo!("This should never happen"),
            };
            self.next();
            left = Node::BinOp(Box::new(left), op, Box::new(self.factor()));
        }
        return left;
    }

    fn expr(&mut self) -> Node {
        let mut left = self.term();

        loop {
            match self.cur_tok {
                Token::Plus | Token::Minus => {}
                _ => break,
            }
            let op = match self.cur_tok {
                Token::Plus => Operation::Plus,
                Token::Minus => Operation::Minus,
                _ => todo!("This should never happen"),
            };
            self.next();
            left = Node::BinOp(Box::new(left), op, Box::new(self.term()));
        }
        return left;
    }

    pub fn build_tree(&mut self) -> Node {
        return self.expr();
    }
}

#[derive(Debug, Clone)]
pub enum CalculatorValue {
    String(String),
    Number(f64),
    Range(Position, Position),
}

impl CalculatorValue {
    pub fn to_f64(&self, symbols: &mut HashMap<String, CalculatorValue>, table: &Table) -> f64 {
        match self {
            CalculatorValue::String(..) => 0.0,
            CalculatorValue::Number(n) => *n,
            CalculatorValue::Range(start, end) => {
                let vals = table.get_values_at_range(&start, &end);
                let mut sum = 0.0;
                for val in vals {
                    match val {
                        table::Data::String(..) => sum += 0.0,
                        table::Data::Number(n) => sum += n.parse().unwrap_or(0.0),
                        table::Data::Equation(e, cache) => {
                            if let Some(r) = cache {
                                sum += r.to_f64(symbols, table);
                            } else {
                                if let Ok(v) = calculate(&e, symbols, table){
                                    sum += v.to_f64(symbols, table);
                                }
                            }
                        }
                    }
                }
                return sum;
            }
        }
    }
}

pub struct Interpreter {
    tree: Node,
}

impl Interpreter {
    pub fn new(tree: Node) -> Interpreter {
        return Interpreter { tree };
    }

    pub fn interpret(
        &self,
        symbols: &mut HashMap<String, CalculatorValue>,
        table: &Table,
    ) -> Result<CalculatorValue, u8> {
        let rec_count = symbols.get_mut("%recursion");
        if let Some(CalculatorValue::Number(n)) = rec_count {
            *n += 1.0;
            if *n > RECURSION_LIMIT{
                eprintln!("rec: {:?}", rec_count);
                return Err(1)
            }
        }
        self.tree.visit(symbols, table)
    }
}

pub fn get_tokens(equation: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(equation);
    let toks = lexer.tokenize();
    return toks;
}

pub fn calcualte_from_tokens(
    tokens: Vec<Token>,
    symbols: &mut HashMap<String, CalculatorValue>,
    table: &Table,
) -> Result<CalculatorValue, u8> {
    let mut parser = Parser::new(tokens);
    let tree = parser.build_tree();
    let int = Interpreter::new(tree);
    let val = int.interpret(symbols, table);
    return val;
}

pub fn calculate(
    equation: &str,
    symbols: &mut HashMap<String, CalculatorValue>,
    table: &Table,
) -> Result<CalculatorValue, u8> {
    let toks = get_tokens(equation);
    return calcualte_from_tokens(toks, symbols, table);
}
