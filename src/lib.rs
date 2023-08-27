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

enum CompoundTypeMarker {
    Message,
    Oneof,
}

enum CompoundType {
    Message(Message),
    Oneof(Oneof),
}

impl Parser {
    fn new(input: &str) -> Self {
        Parser {
            body: str_to_vec(input),
            curr_index: 0,
        }
    }
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

        match t {
            Type::String => MessageField::String(frequency, name, position),
            Type::Int32 => MessageField::Int32(frequency, name, position),
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

    fn is_oneof(&self) -> bool {
        self.matches("oneof")
    }

    fn consume_oneof(&mut self) -> Oneof {
        let oneof = self.consume_compound_type(CompoundTypeMarker::Oneof);
        match oneof {
            CompoundType::Oneof(oneof) => oneof,
            _ => unreachable!(),
        }
    }

    fn consume_message(&mut self) -> Message {
        let message = self.consume_compound_type(CompoundTypeMarker::Message);

        match message {
            CompoundType::Message(message) => message,
            _ => unreachable!(),
        }
    }

    fn consume_compound_type(&mut self, marker: CompoundTypeMarker) -> CompoundType {
        self.skip_whitespace_or_comment();
        match marker {
            CompoundTypeMarker::Oneof => self.consume("oneof"),
            CompoundTypeMarker::Message => self.consume("message"),
        };
        self.skip_whitespace_or_comment();
        let name = self.consume_name();
        self.skip_whitespace_or_comment();
        self.skip('{');
        self.skip_whitespace_or_comment();

        let mut enums = vec![];
        let mut fields = vec![];
        let mut messages = vec![];
        let mut oneofs = vec![];

        while self.is_message_field() || self.is_enum() || self.is_message() || self.is_oneof() {
            self.skip_whitespace_or_comment();

            if self.is_message_field() {
                fields.push(self.consume_message_field());
            } else if self.is_enum() {
                enums.push(self.consume_enum());
            } else if self.is_message() {
                messages.push(self.consume_message());
            } else if self.is_oneof() {
                oneofs.push(self.consume_oneof());
            } else {
                unreachable!();
            }
            self.skip_whitespace_or_comment();
        }

        self.skip_whitespace_or_comment();
        self.skip('}');
        self.skip_whitespace_or_comment();

        match marker {
            CompoundTypeMarker::Message => CompoundType::Message(Message {
                name,
                messages,
                enums,
                fields,
                oneofs,
            }),
            CompoundTypeMarker::Oneof => CompoundType::Oneof(Oneof {
                name,
                messages,
                enums,
                fields,
                oneofs,
            }),
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
    messages: Vec<Message>,
    enums: Vec<Enum>,
    fields: Vec<MessageField>,
    oneofs: Vec<Oneof>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
struct Oneof {
    name: String,
    messages: Vec<Message>,
    enums: Vec<Enum>,
    fields: Vec<MessageField>,
    oneofs: Vec<Oneof>,
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
enum MessageField {
    Int32(Option<Frequency>, String, u32),
    String(Option<Frequency>, String, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

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

                message second_inner_inner {
                    repeated int32 inner_field = 1;

                    enum inner_inner_inner_enum {
                        one = 1;
                        two = 2;
                    }
                }
            }
        }
//  xd";
        let mut parser = Parser::new(input);

        let output = parser.consume_message();
        let expected = Message {
            name: "blah".to_string(),
            oneofs: vec![],
            messages: vec![Message {
                name: "inner".to_string(),
                oneofs: vec![],
                messages: vec![
                    Message {
                        name: "inner_inner".to_string(),
                        messages: vec![],
                        enums: vec![Enum {
                            name: "inner_inner_enum".to_string(),
                            fields: vec![EnumField {
                                name: "one".to_string(),
                                position: 1,
                            }],
                        }],
                        fields: vec![
                            MessageField::String(
                                Some(Frequency::Optional),
                                "inner_inner_field".to_string(),
                                1,
                            ),
                            MessageField::Int32(
                                Some(Frequency::Repeated),
                                "second_inner_inner_field".to_string(),
                                2,
                            ),
                        ],
                        oneofs: vec![],
                    },
                    Message {
                        name: "second_inner_inner".to_string(),
                        messages: vec![],
                        enums: vec![Enum {
                            name: "inner_inner_inner_enum".to_string(),
                            fields: vec![
                                EnumField {
                                    name: "one".to_string(),
                                    position: 1,
                                },
                                EnumField {
                                    name: "two".to_string(),
                                    position: 2,
                                },
                            ],
                        }],
                        fields: vec![MessageField::Int32(
                            Some(Frequency::Repeated),
                            "inner_field".to_string(),
                            1,
                        )],
                        oneofs: vec![],
                    },
                ],
                enums: vec![Enum {
                    name: "inner_enum".to_string(),
                    fields: vec![EnumField {
                        name: "one".to_string(),
                        position: 1,
                    }],
                }],
                fields: vec![
                    MessageField::String(Some(Frequency::Optional), "inner_field".to_string(), 1),
                    MessageField::Int32(
                        Some(Frequency::Repeated),
                        "second_inner_field".to_string(),
                        2,
                    ),
                ],
            }],
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
                MessageField::Int32(Some(Frequency::Repeated), "first".to_string(), 1),
                MessageField::String(Some(Frequency::Repeated), "second".to_string(), 2),
                MessageField::String(Some(Frequency::Optional), "third".to_string(), 3),
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
        let mut parser = Parser::new(input);

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
        let mut parser = Parser::new(input);

        let res = parser.consume_message();

        assert!(parser.is_finished());

        assert_eq!(
            res,
            Message {
                name: "Person".to_string(),
                messages: vec![],
                enums: vec![],
                fields: vec![
                    MessageField::String(None, "name".to_string(), 1),
                    MessageField::Int32(None, "id".to_string(), 2),
                    MessageField::Int32(Some(Frequency::Required), "age".to_string(), 3)
                ],
                oneofs: vec![],
            },
        );
    }

    #[test]
    fn parse_oneof() {
        let input = "message SampleMessage {
  oneof test_oneof {
    string name = 4;
    int32 sub_message = 9;
  }
  }";

        let mut parser = Parser::new(input);

        let res = parser.consume_message();

        assert!(parser.is_finished());

        assert_eq!(
            res,
            Message {
                name: "SampleMessage".to_string(),
                messages: vec![],
                enums: vec![],
                fields: vec![],
                oneofs: vec![Oneof {
                    name: "test_oneof".to_string(),
                    fields: vec![
                        MessageField::String(None, "name".to_string(), 4),
                        MessageField::Int32(None, "sub_message".to_string(), 9)
                    ],
                    messages: vec![],
                    enums: vec![],
                    oneofs: vec![]
                }]
            }
        );
    }
}
