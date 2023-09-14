use std::fs;

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

struct Lexer {
    tokens: Vec<Token>,
    contents: String,
    position: usize,
}

impl Lexer {
    pub fn next(&mut self) -> bool {
        self.position += 1;
        if self.position >= self.contents.len() {
            return false;
        }
        return true;
    }

    pub fn back(&mut self) {
        self.position -= 1;
    }

    pub fn get_char(&self) -> char {
        return self.contents.chars().nth(self.position).unwrap();
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
    while lexer.next() {
        let char = lexer.get_char();
        if char == '"' {
            break;
        }
        str += &char.to_string();
    }
    return str;
}

fn parse_number(lexer: &mut Lexer) -> f64 {
    let mut text = lexer.get_char().to_string();
    let mut is_dec = false;
    while lexer.next() {
        let char = lexer.get_char();
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

    lexer.back();

    let n: f64 = text.parse().expect("Could not parse number");
    return n;
}

fn parse_expr(lexer: &mut Lexer) -> String{
    let mut str = String::new();
    let mut paren_count = 1;
    while lexer.next() {
        let char = lexer.get_char();
        if char == ')' {
            paren_count -= 1;
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
        contents: contents.to_string(),
        position: 0,
    };

    loop {
        let char = lexer.get_char();
        if char != ' ' && char != '\n' && char != '\t' && char != '\r' {
            let tok = match char {
                ']' => Token::RBracket,
                '[' => Token::LBracket,
                '(' => Token::Expr(parse_expr(&mut lexer)),
                '"' => Token::String(parse_string(&mut lexer)),
                ',' => Token::Comma,
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    Token::Number(parse_number(&mut lexer))
                }
                _ => Token::Err(char),
            };
            lexer.add_token(tok);
        }

        if !lexer.next() {
            break;
        }
    }

    return lexer.get_tokens();
}
