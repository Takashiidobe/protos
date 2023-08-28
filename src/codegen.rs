use crate::*;

trait PrettyPrint {
    fn pretty_print(&self, depth: u64) -> String;
}

fn indent_string(s: &mut String, depth: u64) {
    for _ in 0..depth {
        s.push('\t');
    }
}

impl PrettyPrint for Message {
    fn pretty_print(&self, depth: u64) -> String {
        let mut s = String::default();

        let Message {
            name,
            messages,
            enums,
            fields,
            oneofs,
        } = self;

        s.push_str(&format!("message {} {{\n", name));

        for field in fields {
            s.push_str(&field.pretty_print(depth));
        }

        for e in enums {
            s.push_str(&e.pretty_print(depth + 1));
        }

        for message in messages {
            s.push_str(&message.pretty_print(depth + 1));
        }

        for oneof in oneofs {
            s.push_str(&oneof.pretty_print(depth + 1));
        }

        s.push('}');

        s
    }
}

impl PrettyPrint for Oneof {
    fn pretty_print(&self, depth: u64) -> String {
        let mut s = String::default();

        let Oneof {
            name,
            messages,
            enums,
            fields,
            oneofs,
        } = self;

        s.push_str(&format!("oneof {} {{\n", name));

        for field in fields {
            s.push_str(&field.pretty_print(depth + 1));
        }

        for e in enums {
            s.push_str(&e.pretty_print(depth + 1));
        }

        for message in messages {
            s.push_str(&message.pretty_print(depth + 1));
        }

        for oneof in oneofs {
            s.push_str(&oneof.pretty_print(depth + 1));
        }

        s.push('}');

        s
    }
}

impl PrettyPrint for Enum {
    fn pretty_print(&self, depth: u64) -> String {
        let mut s = String::default();

        let Enum { name, fields } = self;

        s.push_str(&format!("enum {} {{\n", name));

        for field in fields {
            s.push_str(&field.pretty_print(depth + 1));
        }

        s.push('}');

        s
    }
}

impl PrettyPrint for EnumField {
    fn pretty_print(&self, depth: u64) -> String {
        let EnumField { name, position } = self;

        let mut s = String::default();

        indent_string(&mut s, depth);

        s.push_str(&format!("{} = {};\n", name, position));

        s
    }
}

impl PrettyPrint for MessageField {
    fn pretty_print(&self, depth: u64) -> String {
        let mut s = String::default();
        indent_string(&mut s, depth);

        let rest = match self {
            MessageField::Int32(frequency, name, position) => {
                if let Some(freq) = frequency {
                    format!(
                        "{} int32 {} {};\n",
                        std::convert::Into::<String>::into(freq.clone()),
                        name,
                        position
                    )
                } else {
                    format!("int32 {} {};\n", name, position)
                }
            }
            MessageField::String(frequency, name, position) => {
                if let Some(freq) = frequency {
                    format!(
                        "{} string {} {};\n",
                        std::convert::Into::<String>::into(freq.clone()),
                        name,
                        position
                    )
                } else {
                    format!("string {} {};\n", name, position)
                }
            }
        };

        s.push_str(&rest);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen() {
        let input = Message {
            name: "SampleMessage".to_string(),
            messages: vec![],
            enums: vec![],
            fields: vec![],
            oneofs: vec![Oneof {
                name: "test_oneof".to_string(),
                fields: vec![
                    MessageField::String(None, "name".to_string(), 4),
                    MessageField::Int32(None, "sub_message".to_string(), 9),
                ],
                messages: vec![],
                enums: vec![],
                oneofs: vec![],
            }],
        };

        assert_eq!(
            input.pretty_print(0),
            "message SampleMessage {\noneof test_oneof {\n\t\tstring name 4;\n\t\tint32 sub_message 9;\n}}"
        );
    }

    #[test]
    fn basic_message() {
        let input = Message {
            name: "Person".to_string(),
            messages: vec![],
            enums: vec![],
            fields: vec![
                MessageField::String(None, "name".to_string(), 1),
                MessageField::Int32(None, "id".to_string(), 2),
                MessageField::Int32(Some(Frequency::Required), "age".to_string(), 3),
            ],
            oneofs: vec![],
        };

        assert_eq!(
            input.pretty_print(0),
            "message Person {\nstring name 1;\nint32 id 2;\nrequired int32 age 3;\n}"
        );
    }

    #[test]
    fn enum_f() {
        let input = Enum {
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
        };

        assert_eq!(
            input.pretty_print(0),
            "enum Person {\n\tname = 1;\n\tid = 2;\n\tage = 3;\n}"
        );
    }

    #[test]
    fn recursive_message() {
        let input = Message {
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

        assert_eq!(input.pretty_print(0),
        "message blah {\nrepeated int32 first 1;\nrepeated string second 2;\noptional string third 3;\nenum Person {\n\t\tname = 1;\n\t\tid = 2;\n\t\tage = 3;\n}enum Other {\n\t\tone = 1;\n\t\ttwo = 2;\n\t\tthree = 3;\n}message inner {\n\toptional string inner_field 1;\n\trepeated int32 second_inner_field 2;\nenum inner_enum {\n\t\t\tone = 1;\n}message inner_inner {\n\t\toptional string inner_inner_field 1;\n\t\trepeated int32 second_inner_inner_field 2;\nenum inner_inner_enum {\n\t\t\t\tone = 1;\n}}message second_inner_inner {\n\t\trepeated int32 inner_field 1;\nenum inner_inner_inner_enum {\n\t\t\t\tone = 1;\n\t\t\t\ttwo = 2;\n}}}}");
    }
}
