pub struct CommandLine {
    column: usize,
    line: usize,
    current_text: Option<String>,
    result_text: Option<String>,
}

impl CommandLine {
    pub fn new(column: usize, line: usize) -> CommandLine {
        CommandLine {
            column,
            line,
            current_text: None,
            result_text: None,
        }
    }

    pub fn get_current_text(&self) -> &str {
        match &self.current_text {
            None => "",
            Some(t) => t,
        }
    }

    pub fn remove_last_char(&mut self) {
        match &self.current_text {
            Some(t) => {
                if t.len() != 0 {
                    self.current_text = Some(String::from(&t[0..t.len() - 1]))
                }
            }
            _ => {}
        }
    }

    pub fn clear_text(&mut self) {
        self.current_text = None;
        self.result_text = None;
    }

    pub fn add_text_to_current_command(&mut self, text: &str) {
        self.current_text = Some(match &self.current_text {
            &None => String::from(text),
            Some(t) => t.to_owned() + &text,
        });
    }

    pub fn display(&self) -> String {
        let ttd = match (&self.current_text, &self.result_text) {
            (Some(t), ..) => t.to_owned(),
            (None, Some(t)) => t.to_owned(),
            _ => "".to_string()
        };
        format!(
            "\x1b[s\x1b[{};{}H{}\x1b[u",
            self.line, self.column, ttd
        )
    }

    pub fn print(&mut self, text: &str) {
        self.result_text = Some(String::from(text));
    }
}
