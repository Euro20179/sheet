#[derive(Debug)]
pub enum Token {
    Col(String),
    Row(usize),
}

pub struct Lexer {
    contents: String,
    cur_pos: usize,
    cur_char: char,
}

fn to_lower(ch: char) -> char {
    let mut ch_byte = ch as u8;
    ch_byte += 32;
    return ch_byte as char;
}

impl Lexer {
    pub fn new(contents: String) -> Lexer {
        Lexer {
            cur_pos: 0,
            cur_char: contents.chars().nth(0).unwrap(),
            contents,
        }
    }

    fn next(&mut self) -> bool {
        self.cur_pos += 1;
        if self.cur_pos >= self.contents.len() {
            return false;
        }
        self.cur_char = self.contents.chars().nth(self.cur_pos).unwrap();
        return true;
    }

    fn back(&mut self) {
        if self.cur_pos > 0 {
            self.cur_pos -= 1;
            self.cur_char = self.contents.chars().nth(self.cur_pos).unwrap();
        }
    }

    fn build_col_repr(&mut self) -> String {
        let mut repr = String::from(self.cur_char);

        while self.next() {
            let ch = match self.cur_char {
                'A'..='Z' => to_lower(self.cur_char),
                'a'..='z' => self.cur_char,
                _ => {
                    self.back();
                    break;
                }
            };

            repr += &String::from(ch);
        }

        return repr;
    }

    fn build_row_repr(&mut self) -> usize {
        let mut repr = String::from(self.cur_char);

        while self.next() {
            let ch = match self.cur_char {
                '0'..='9' => self.cur_char,
                _ => {
                    self.back();
                    break
                }
            };

            repr += &String::from(ch);
        }

        return repr.parse::<usize>().unwrap();
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];


        loop {
            let tok = match self.cur_char {
                'A'..='Z' | 'a'..='z' => Token::Col(self.build_col_repr()),
                '0'..='9' => Token::Row(self.build_row_repr()),
                _ => panic!("Invalid position tok")
            };

            tokens.push(tok);

            if !self.next() {
                break;
            }
        }
        return tokens;
    }
}
