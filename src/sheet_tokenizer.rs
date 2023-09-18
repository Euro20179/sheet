use std::{fs, str::Chars};

#[derive(Clone, Debug)]
pub enum Token {
    LBracket,
    RBracket,
    LParen,
    RParen,
    String(String),
    Expr(String),
    Number(f64),
    Err(char),
    Comma,
}

struct Lexer<'a> {
    tokens: Vec<Token>,
    chars: Chars<'a>,
    cur_char: Option<char>,
}

impl Lexer<'_> {
    pub fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        self.cur_char = c;
        return c;
    }

    pub fn get_cur_char(&self) -> Option<char> {
        return self.cur_char;
    }

    pub fn add_token(&mut self, tok: Token) {
        self.tokens.push(tok);
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        return self.tokens.clone();
    }
}

fn parse_string(lexer: &mut Lexer) -> String {
    let mut str = String::new();
    while let Some(ch) = lexer.next() {
        if ch == '"' {
            break;
        }
        str += &ch.to_string();
    }
    return str;
}

fn parse_number(lexer: &mut Lexer, start_char: char) -> f64 {
    let mut text = start_char.to_string();
    let mut is_dec = false;
    while let Some(char) = lexer.next() {
        if char == '.' && !is_dec {
            is_dec = true;
            text += &char.to_string();
        } else if char == '.' {
            break;
        } else if let '0'..='9' = char {
            text += &char.to_string();
        } else {
            break;
        }
    }

    let n: f64 = text.parse().expect("Could not parse number");
    return n;
}

fn parse_expr(lexer: &mut Lexer) -> String {
    let mut str = String::new();
    let mut paren_count = 1;
    while let Some(char) = lexer.next() {
        if char == ')' {
            paren_count -= 1;
        } else if char == '(' {
            paren_count += 1;
        }
        if paren_count == 0 {
            break;
        }
        str += &char.to_string();
    }
    return str;
}

pub fn parse(contents: &str) -> Vec<Token> {
    let mut lexer = Lexer {
        tokens: vec![],
        chars: contents.chars(),
        cur_char: None,
    };

    //number eats 1 character too far, after the call to parse_number, do not call lexer.next
    let mut get_next = true;
    loop {
        let cur_char: Option<char>;
        if get_next {
            cur_char = lexer.next();
        } else if let Some(ch) = lexer.get_cur_char() {
            get_next = true;
            cur_char = Some(ch);
        } else {
            break;
        }
        match cur_char {
            None => break,
            Some(char) => {
                let tok = match char {
                    ' ' | '\n' | '\t' | '\r' => continue,
                    ']' => Token::RBracket,
                    '[' => Token::LBracket,
                    '(' => Token::Expr(parse_expr(&mut lexer)),
                    '"' => Token::String(parse_string(&mut lexer)),
                    ',' => Token::Comma,
                    '0'..='9' => {
                        get_next = false;
                        Token::Number(parse_number(&mut lexer, char))
                    }
                    _ => Token::Err(char),
                };
                lexer.add_token(tok);
            }
        }
    }

    return lexer.get_tokens();
}
