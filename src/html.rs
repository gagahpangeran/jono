use crate::{css, dom, parser, style};

pub fn parse(source: String) -> Document {
    let mut nodes = parser::Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();

    let root_node = if nodes.len() == 1 {
        nodes.remove(0)
    } else {
        dom::elem("html".to_string(), dom::AttrMap::new(), nodes)
    };

    let mut stylesheets = vec![css::user_agent_stylesheet()];
    stylesheets.extend(get_author_stylesheets(&root_node));

    Document {
        root_node,
        stylesheets,
    }
}

fn get_author_stylesheets(root: &dom::Node) -> Vec<css::Stylesheet> {
    root.find_elements("style")
        .into_iter()
        .map(|node| css::parse(node.text_content()))
        .collect()
}

pub struct Document {
    pub root_node: dom::Node,
    pub stylesheets: Vec<css::Stylesheet>,
}

impl Document {
    pub fn style_tree(&self) -> style::StyleNode<'_> {
        style::style_tree(&self.root_node, &self.stylesheets)
    }
}

impl parser::Parser {
    fn parse_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    fn parse_node(&mut self) -> dom::Node {
        if self.starts_with("<!--") {
            self.parse_comment()
        } else if self.starts_with("<") {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    fn parse_comment(&mut self) -> dom::Node {
        self.expect("<!--");

        let mut text = String::new();
        while !self.eof() && !self.starts_with("-->") {
            text.push(self.consume_char());
        }

        self.expect("-->");

        dom::comment(text)
    }

    fn parse_element(&mut self) -> dom::Node {
        self.expect("<");
        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();
        self.expect(">");

        let children = self.parse_nodes();

        self.expect("</");
        self.expect(&tag_name);
        self.expect(">");

        return dom::elem(tag_name, attrs, children);
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_name();
        self.expect("=");
        let value = self.parse_attr_value();
        (name, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        value
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = dom::AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}
