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
}

impl Lexer<'_> {
    pub fn next(&mut self) -> Option<char> {
        return self.chars.next();
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
        } else if char == '.' {
            break;
        } else if match char {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => false,
            _ => true,
        } {
            break;
        }
        text += &char.to_string();
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
    };

    loop {
        let char = lexer.next();
        match char {
            None => break,
            Some(char) => {
                if char != ' ' && char != '\n' && char != '\t' && char != '\r' {
                    let tok = match char {
                        ']' => Token::RBracket,
                        '[' => Token::LBracket,
                        '(' => Token::Expr(parse_expr(&mut lexer)),
                        '"' => Token::String(parse_string(&mut lexer)),
                        ',' => Token::Comma,
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            Token::Number(parse_number(&mut lexer, char))
                        }
                        _ => Token::Err(char),
                    };
                    lexer.add_token(tok);
                }
            }
        }
    }

    return lexer.get_tokens();
}
