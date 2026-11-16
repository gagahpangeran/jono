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
        self.specified_values.get(name).cloned()
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

pub fn style_tree<'a>(root: &'a dom::Node, stylesheets: &'a [css::Stylesheet]) -> StyleNode<'a> {
    StyleNode {
        node: root,
        specified_values: match root.node_type {
            dom::NodeType::Element(ref elem) => specified_values(elem, stylesheets),
            dom::NodeType::Comment(_) => PropertyMap::from([(
                "display".to_string(),
                css::Value::Keyword("none".to_string()),
            )]),
            _ => PropertyMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheets))
            .collect(),
    }
}

type MatchedRule<'a> = (css::Origin, css::Specificity, &'a css::Rule);

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

    true
}

fn match_rule<'a>(
    elem: &dom::ElementData,
    rule: &'a css::Rule,
    origin: css::Origin,
) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (origin, selector.specificity(), rule))
}

fn matching_rules<'a>(
    elem: &dom::ElementData,
    stylesheets: &'a [css::Stylesheet],
) -> Vec<MatchedRule<'a>> {
    stylesheets
        .iter()
        .flat_map(|stylesheet| {
            stylesheet
                .rules
                .iter()
                .filter_map(|rule| match_rule(elem, rule, stylesheet.origin))
        })
        .collect()
}

fn specified_values(elem: &dom::ElementData, stylesheets: &[css::Stylesheet]) -> PropertyMap {
    let mut values = PropertyMap::new();
    let mut rules = matching_rules(elem, stylesheets);

    rules.sort_by(|&(orig_a, spec_a, _), &(orig_b, spec_b, _)| {
        (orig_a, spec_a).cmp(&(orig_b, spec_b))
    });

    for (_, _, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}
