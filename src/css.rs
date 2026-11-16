use crate::parser;

#[derive(Debug)]
pub struct Stylesheet {
    pub origin: Origin,
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Copy for Color {}

pub type Specificity = (usize, usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    UserAgent,
    Author,
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0,
        }
    }
}

pub fn parse(source: String) -> Stylesheet {
    let mut parser = parser::Parser {
        pos: 0,
        input: source,
    };
    Stylesheet {
        origin: Origin::Author,
        rules: parser.parse_rules(),
    }
}

pub fn user_agent_stylesheet() -> Stylesheet {
    let mut stylesheet = parse(include_str!("user-agent.css").to_string());
    stylesheet.origin = Origin::UserAgent;
    stylesheet
}

fn valid_identifier_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

impl parser::Parser {
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }

        selectors.sort_by_key(|s| s.specificity());
        selectors
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        self.expect("{");
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        self.expect(":");
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        self.expect(";");

        Declaration { name, value }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        self.consume_while(|c| matches!(c, '0'..='9' | '.'))
            .parse()
            .unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        self.expect("#");
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }
}
