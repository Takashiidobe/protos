pub struct Parser {
    body: Vec<char>,
    curr_index: usize,
}

fn str_to_vec(s: &str) -> Vec<char> {
    s.chars().collect()
}

impl Parser {
    fn parse(&self) -> Field {
        Field::Message("".to_string(), None)
    }

    fn peek_curr(&self) -> char {
        self.body[self.curr_index]
    }

    fn peek_next(&self) -> char {
        self.body[self.curr_index + 1]
    }

    fn consume(&mut self, s: &str) -> String {
        let veced_str = str_to_vec(s);
        let veced_str_len = veced_str.len();
        if self.body[self.curr_index..self.curr_index + veced_str_len] == veced_str {
            self.curr_index += veced_str_len;
            return s.to_string();
        }
        panic!("Could not consume");
    }

    fn consume_name(&mut self) -> String {
        self.skip_whitespace();
        let mut ret = String::new();
        while self.body[self.curr_index].is_ascii_alphabetic() {
            ret.push(self.body[self.curr_index]);
            self.curr_index += 1;
        }
        ret
    }

    fn skip(&mut self, c: char) -> bool {
        if self.body[self.curr_index] == c {
            self.curr_index += 1;
            return true;
        }
        false
    }

    fn is_in_bounds(&self) -> bool {
        self.curr_index < self.body.len()
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut ret = false;
        while self.is_in_bounds() && self.curr_char().is_ascii_whitespace() {
            self.curr_index += 1;
            ret = true;
        }
        ret
    }

    fn skip_comment(&mut self) {
        if self.peek_next() == '/' && self.peek_curr() == '/' {
            self.skip_until('\n');
        }
    }

    fn skip_until(&mut self, c: char) {
        while self.peek_curr() != c {
            self.curr_index += 1;
        }
    }

    fn matches(&self, s: &str) -> bool {
        let veced_str = str_to_vec(s);
        let veced_str_len = veced_str.len();

        if self.curr_index + veced_str_len > self.body.len() {
            return false;
        }

        self.body[self.curr_index..self.curr_index + veced_str_len] == veced_str
    }

    fn consume_number(&mut self) -> u32 {
        let mut res = 0;
        while self.curr_char().is_ascii_digit() {
            res *= 10;
            res += self.curr_char().to_digit(10).unwrap();
            self.curr_index += 1;
        }
        res
    }

    fn curr_char(&self) -> char {
        self.body[self.curr_index]
    }

    fn consume_any(&mut self, choices: Vec<&str>) -> String {
        for choice in choices {
            if self.matches(choice) {
                return self.consume(choice);
            }
        }

        panic!("Could not consume any");
    }

    fn is_finished(&self) -> bool {
        self.curr_index == self.body.len()
    }
}

pub type MessageName = String;

#[derive(Debug, Clone)]
enum Frequency {
    Optional,
    Repeated,
    Required,
}

#[derive(Debug, Clone)]
enum Type {
    String(String),
    I32(i32),
}

#[derive(Debug, Clone)]
struct Field {
    t: Type,
    name: String,
    position: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        let input = " message   Person {
            string name = 1; // name of person
            int32 id = 2;  // Unique ID number for this person.
    }  ";
        let mut parser = Parser {
            body: input.chars().collect(),
            curr_index: 0,
        };

        parser.skip_whitespace();
        parser.consume("message");
        let name = parser.consume_name();
        parser.skip_whitespace();
        parser.skip('{');
        parser.skip_whitespace();

        let message_type = parser.consume_any(vec!["string", "int32"]);
        parser.skip_whitespace();
        let message_name = parser.consume_name();
        parser.skip_whitespace();
        parser.consume("=");
        parser.skip_whitespace();
        let field_num = parser.consume_number();
        parser.skip_whitespace();
        parser.consume(";");
        parser.skip_whitespace();
        parser.skip_comment();
        parser.skip_whitespace();

        let message_type_2 = parser.consume_any(vec!["string", "int32"]);
        parser.skip_whitespace();
        let message_name_2 = parser.consume_name();
        parser.skip_whitespace();
        parser.consume("=");
        parser.skip_whitespace();
        let field_num_2 = parser.consume_number();
        parser.skip_whitespace();
        parser.consume(";");
        parser.skip_whitespace();
        parser.skip_comment();
        parser.skip_whitespace();

        parser.skip('}');
        parser.skip_whitespace();

        assert_eq!(name, "Person");
        assert!(parser.is_finished());
        assert_eq!(message_type, "string");
        assert_eq!(message_name, "name");
        assert_eq!(field_num, 1);
        assert_eq!(message_type_2, "int32");
        assert_eq!(message_name_2, "id");
        assert_eq!(field_num_2, 2);
    }
}
