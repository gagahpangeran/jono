use std::collections::{HashMap, HashSet};

pub type AttrMap = HashMap<String, String>;

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(String),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attrs: AttrMap,
}

pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

pub fn elem(tag_name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData { tag_name, attrs }),
    }
}

pub fn comment(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Comment(data),
    }
}

impl Node {
    pub fn find_elements(&self, tag: &str) -> Vec<&Node> {
        let mut nodes = Vec::new();

        if matches!(&self.node_type, NodeType::Element(e) if e.tag_name == tag) {
            nodes.push(self);
        }

        for child in &self.children {
            nodes.extend(child.find_elements(tag));
        }

        nodes
    }

    pub fn text_content(&self) -> String {
        match self.node_type {
            NodeType::Text(ref t) => t.clone(),
            _ => self.children.iter().map(Node::text_content).collect(),
        }
    }
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attrs.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attrs.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}
