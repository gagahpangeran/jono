use std::collections::HashMap;

use crate::{css, dom};

pub type PropertyMap = HashMap<String, css::Value>;

#[derive(Debug)]
pub struct StyleNode<'a> {
    pub node: &'a dom::Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyleNode<'a>>,
}

#[derive(PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyleNode<'a> {
    pub fn value(&self, name: &str) -> Option<css::Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(css::Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &css::Value) -> css::Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }
}

fn matches(elem: &dom::ElementData, selector: &css::Selector) -> bool {
    match selector {
        css::Selector::Simple(s) => matches_simple_selector(elem, s),
    }
}

fn matches_simple_selector(elem: &dom::ElementData, selector: &css::SimpleSelector) -> bool {
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    if selector
        .class
        .iter()
        .any(|class| !elem.classes().contains(class.as_str()))
    {
        return false;
    }

    return true;
}

type MatchedRule<'a> = (css::Specificity, &'a css::Rule);

fn match_rule<'a>(elem: &dom::ElementData, rule: &'a css::Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matching_rules<'a>(
    elem: &dom::ElementData,
    stylesheet: &'a css::Stylesheet,
) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

fn specified_values(elem: &dom::ElementData, stylesheet: &css::Stylesheet) -> PropertyMap {
    let mut values = PropertyMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

pub fn style_tree<'a>(root: &'a dom::Node, stylesheet: &'a css::Stylesheet) -> StyleNode<'a> {
    StyleNode {
        node: root,
        specified_values: match root.node_type {
            dom::NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            _ => PropertyMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}
