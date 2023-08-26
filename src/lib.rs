#[derive(Debug)]
pub struct Parser {
    body: Vec<char>,
    curr_index: usize,
}

fn str_to_vec(s: &str) -> Vec<char> {
    s.chars().collect()
}

const TYPES: &[&str] = &["string", "int32"];

const FREQUENCIES: &[&str] = &["optional", "repeated", "required"];

impl Parser {
    fn peek_curr(&self) -> Option<char> {
        if self.is_in_bounds() {
            Some(self.body[self.curr_index])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.next_is_in_bounds() {
            Some(self.body[self.curr_index + 1])
        } else {
            None
        }
    }

    fn next_is_in_bounds(&self) -> bool {
        self.curr_index + 1 < self.body.len()
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
        self.skip_whitespace_or_comment();
        let mut ret = String::new();
        while self.curr_char().is_ascii_alphabetic() || self.curr_char() == '_' {
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

    fn is_whitespace(&self) -> bool {
        self.is_in_bounds() && self.curr_char().is_ascii_whitespace()
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut ret = false;
        while self.is_whitespace() {
            self.curr_index += 1;
            ret = true;
        }
        ret
    }

    fn skip_comment(&mut self) -> bool {
        if self.is_comment() {
            self.skip_until('\n');
            return true;
        }
        false
    }

    fn is_comment(&self) -> bool {
        if let (Some(curr), Some(next)) = (self.peek_curr(), self.peek_next()) {
            curr == '/' && next == '/'
        } else {
            false
        }
    }

    fn skip_until(&mut self, c: char) {
        while let Some(curr) = self.peek_curr() {
            if curr != c {
                self.curr_index += 1;
            } else {
                break;
            }
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

    fn consume_message_field(&mut self) -> MessageField {
        self.skip_whitespace_or_comment();
        let frequency = self.consume_frequency();
        self.skip_whitespace_or_comment();
        let t = self.consume_type();
        self.skip_whitespace_or_comment();
        let name = self.consume_name();
        self.skip_whitespace_or_comment();
        self.skip('=');
        self.skip_whitespace_or_comment();
        let position = self.consume_number();
        self.skip_whitespace_or_comment();
        self.skip(';');
        self.skip_whitespace_or_comment();

        MessageField {
            frequency,
            t,
            name,
            position,
        }
    }

    fn consume_message_fields(&mut self) -> Vec<MessageField> {
        self.skip_whitespace_or_comment();
        let mut fields = vec![];
        while self.curr_char() != '}' {
            self.skip_whitespace_or_comment();
            fields.push(self.consume_message_field());
            self.skip_whitespace_or_comment();
        }
        self.skip_whitespace_or_comment();
        fields
    }

    fn skip_whitespace_or_comment(&mut self) {
        while self.is_whitespace() || self.is_comment() {
            self.skip_whitespace();
            self.skip_comment();
        }
    }

    fn consume_any(&mut self, choices: &[&str]) -> Option<String> {
        for choice in choices {
            if self.matches(choice) {
                return Some(self.consume(choice));
            }
        }

        None
    }

    fn consume_type(&mut self) -> Type {
        if let Some(t) = self.consume_any(TYPES) {
            t.into()
        } else {
            panic!("Could not consume type")
        }
    }

    fn consume_frequency(&mut self) -> Option<Frequency> {
        self.consume_any(FREQUENCIES).map(|freq| freq.into())
    }

    fn is_finished(&self) -> bool {
        self.curr_index == self.body.len()
    }

    fn consume_enum(&mut self) -> Enum {
        self.skip_whitespace_or_comment();
        self.consume("enum");
        self.skip_whitespace_or_comment();
        let name = self.consume_name();
        self.skip_whitespace_or_comment();
        self.skip('{');
        self.skip_whitespace_or_comment();
        let fields = self.consume_enum_fields();
        self.skip_whitespace_or_comment();
        self.skip('}');
        self.skip_whitespace_or_comment();

        Enum { name, fields }
    }

    fn is_enum(&self) -> bool {
        self.matches("enum")
    }

    fn matches_any(&self, choices: &[&str]) -> bool {
        for choice in choices {
            if self.matches(choice) {
                return true;
            }
        }

        false
    }

    fn is_message_field(&self) -> bool {
        self.matches_any(&[TYPES, FREQUENCIES].concat())
    }

    fn consume_enum_fields(&mut self) -> Vec<EnumField> {
        self.skip_whitespace_or_comment();
        let mut fields = vec![];
        while self.curr_char() != '}' {
            self.skip_whitespace_or_comment();
            fields.push(self.consume_enum_field());
            self.skip_whitespace_or_comment();
        }
        self.skip_whitespace_or_comment();
        fields
    }

    fn consume_enum_field(&mut self) -> EnumField {
        self.skip_whitespace_or_comment();
        let name = self.consume_name();
        self.skip_whitespace_or_comment();
        self.skip('=');
        self.skip_whitespace_or_comment();
        let position = self.consume_number();
        self.skip_whitespace_or_comment();
        self.skip(';');
        self.skip_whitespace_or_comment();

        EnumField { name, position }
    }

    fn is_message(&self) -> bool {
        self.matches("message")
    }

    fn consume_message(&mut self) -> Message {
        self.skip_whitespace_or_comment();
        self.consume("message");
        self.skip_whitespace_or_comment();
        let name = self.consume_name();
        self.skip_whitespace_or_comment();
        self.skip('{');
        self.skip_whitespace_or_comment();

        let mut enums = vec![];
        let mut fields = vec![];
        let mut inner_message = None;

        while self.is_message_field() || self.is_enum() || self.is_message() {
            self.skip_whitespace_or_comment();
            match (self.is_message_field(), self.is_enum(), self.is_message()) {
                (true, false, false) => fields.push(self.consume_message_field()),
                (false, true, false) => enums.push(self.consume_enum()),
                (false, false, true) => inner_message = Some(Box::new(self.consume_message())),
                _ => unreachable!(),
            }
            self.skip_whitespace_or_comment();
        }

        self.skip_whitespace_or_comment();
        self.skip('}');
        self.skip_whitespace_or_comment();

        Message {
            name,
            inner_message,
            enums,
            fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
enum Frequency {
    Optional,
    Repeated,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
struct Message {
    name: String,
    inner_message: Option<Box<Message>>,
    enums: Vec<Enum>,
    fields: Vec<MessageField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct EnumField {
    name: String,
    position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct Enum {
    name: String,
    fields: Vec<EnumField>,
}

impl From<String> for Frequency {
    fn from(value: String) -> Self {
        match value.as_str() {
            "optional" => Self::Optional,
            "repeated" => Self::Repeated,
            "required" => Self::Required,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
enum Type {
    String,
    Int32,
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        match value.as_str() {
            "string" => Self::String,
            "int32" => Self::Int32,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct MessageField {
    t: Type,
    frequency: Option<Frequency>,
    name: String,
    position: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_parser(input: &str) -> Parser {
        Parser {
            body: input.chars().collect(),
            curr_index: 0,
        }
    }

    #[test]
    fn parse_recursive_message() {
        let input = "
            // haha
            // next

        message blah {

            repeated int32 first = 1;

            enum Person {
                name = 1; // name of person
                id = 2;  // Unique ID number for this person
                age = 3;  // age of person
            }

            enum Other {
                one = 1; // third field
                two = 2;  // second field
                three = 3;  // third field
            }

            repeated string second = 2;
            optional string third = 3;

            message inner {
                optional string inner_field = 1;

                enum inner_enum {
                    one = 1;
                }

                repeated int32 second_inner_field = 2;

                message inner_inner {

                    optional string inner_inner_field = 1;

                    enum inner_inner_enum {
                        one = 1;
                    }

                    repeated int32 second_inner_inner_field = 2;
                }
            }
        }
//  xd";
        let mut parser = setup_parser(input);

        let output = parser.consume_message();

        let expected = Message {
            name: "blah".to_string(),
            inner_message: Some(Box::new(Message {
                name: "inner".to_string(),
                inner_message: Some(Box::new(Message {
                    name: "inner_inner".to_string(),
                    inner_message: None,
                    enums: vec![Enum {
                        name: "inner_inner_enum".to_string(),
                        fields: vec![EnumField {
                            name: "one".to_string(),
                            position: 1,
                        }],
                    }],
                    fields: vec![
                        MessageField {
                            t: Type::String,
                            frequency: Some(Frequency::Optional),
                            name: "inner_inner_field".to_string(),
                            position: 1,
                        },
                        MessageField {
                            t: Type::Int32,
                            frequency: Some(Frequency::Repeated),
                            name: "second_inner_inner_field".to_string(),
                            position: 2,
                        },
                    ],
                })),
                enums: vec![Enum {
                    name: "inner_enum".to_string(),
                    fields: vec![EnumField {
                        name: "one".to_string(),
                        position: 1,
                    }],
                }],
                fields: vec![
                    MessageField {
                        t: Type::String,
                        frequency: Some(Frequency::Optional),
                        name: "inner_field".to_string(),
                        position: 1,
                    },
                    MessageField {
                        t: Type::Int32,
                        frequency: Some(Frequency::Repeated),
                        name: "second_inner_field".to_string(),
                        position: 2,
                    },
                ],
            })),
            enums: vec![
                Enum {
                    name: "Person".to_string(),
                    fields: vec![
                        EnumField {
                            name: "name".to_string(),
                            position: 1,
                        },
                        EnumField {
                            name: "id".to_string(),
                            position: 2,
                        },
                        EnumField {
                            name: "age".to_string(),
                            position: 3,
                        },
                    ],
                },
                Enum {
                    name: "Other".to_string(),
                    fields: vec![
                        EnumField {
                            name: "one".to_string(),
                            position: 1,
                        },
                        EnumField {
                            name: "two".to_string(),
                            position: 2,
                        },
                        EnumField {
                            name: "three".to_string(),
                            position: 3,
                        },
                    ],
                },
            ],
            fields: vec![
                MessageField {
                    t: Type::Int32,
                    frequency: Some(Frequency::Repeated),
                    name: "first".to_string(),
                    position: 1,
                },
                MessageField {
                    t: Type::String,
                    frequency: Some(Frequency::Repeated),
                    name: "second".to_string(),
                    position: 2,
                },
                MessageField {
                    t: Type::String,
                    frequency: Some(Frequency::Optional),
                    name: "third".to_string(),
                    position: 3,
                },
            ],
        };

        assert_eq!(output, expected);
        assert!(parser.is_finished());
    }

    #[test]
    fn parse_enum() {
        let input = "
            // haha
            // next

        enum  Person {
            name = 1; // name of person
            id = 2;  // Unique ID number for this person
            age = 3;  // age of person
    }
//  xd";
        let mut parser = setup_parser(input);

        let output = parser.consume_enum();

        assert!(parser.is_finished());

        assert_eq!(
            output,
            Enum {
                name: "Person".to_string(),
                fields: vec![
                    EnumField {
                        name: "name".to_string(),
                        position: 1
                    },
                    EnumField {
                        name: "id".to_string(),
                        position: 2
                    },
                    EnumField {
                        name: "age".to_string(),
                        position: 3
                    }
                ]
            }
        );
    }

    #[test]
    fn parse_basic_message() {
        let input = "
            // haha
            // next

        message   Person {
            string name = 1; // name of person
            int32 id = 2;  // Unique ID number for this person
            required int32 age = 3;  // age of person
    }
//  xd";
        let mut parser = setup_parser(input);

        let res = parser.consume_message();

        assert!(parser.is_finished());

        assert_eq!(
            res,
            Message {
                name: "Person".to_string(),
                inner_message: None,
                enums: vec![],
                fields: vec![
                    MessageField {
                        t: Type::String,
                        frequency: None,
                        name: "name".to_string(),
                        position: 1
                    },
                    MessageField {
                        t: Type::Int32,
                        frequency: None,
                        name: "id".to_string(),
                        position: 2
                    },
                    MessageField {
                        t: Type::Int32,
                        frequency: Some(Frequency::Required),
                        name: "age".to_string(),
                        position: 3
                    }
                ]
            },
        );
    }
}
