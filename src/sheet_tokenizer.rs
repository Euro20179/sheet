use std::str::Chars;

#[derive(Clone, Debug)]
pub enum Token {
    LBracket,
    RBracket,
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

fn parse_number(lexer: &mut Lexer) -> f64 {
    let mut text = lexer.get_cur_char().unwrap().to_string();
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

    lexer.next();

    loop {
        if let None = lexer.cur_char {
            break;
        } else if let Some(ch) = lexer.cur_char {
            let tok = match ch {
                ']' => Token::RBracket,
                '[' => Token::LBracket,
                ',' => Token::Comma,
                '(' => Token::Expr(parse_expr(&mut lexer)),
                '"' => Token::String(parse_string(&mut lexer)),
                ' ' | '\n' | '\t' | '\r' => {
                    lexer.next();
                    continue;
                }
                '0'..='9' => {
                    let tok = Token::Number(parse_number(&mut lexer));
                    lexer.add_token(tok);
                    continue;
                }
                _ => Token::Err(ch),
            };
            lexer.add_token(tok);
            lexer.next();
        }
    }

    return lexer.get_tokens();
}
