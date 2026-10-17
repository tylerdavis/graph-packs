use markup5ever_rcdom::{Handle, NodeData};

pub fn parse(html: &str) -> Handle {
    htmd::HtmlToMarkdown::new()
        .html_to_tree(html)
        .expect("parsing HTML from a string does not fail")
}

pub fn tag(node: &Handle) -> Option<String> {
    match &node.data {
        NodeData::Element { name, .. } => Some(name.local.to_string()),
        _ => None,
    }
}

pub fn attr(node: &Handle, key: &str) -> Option<String> {
    match &node.data {
        NodeData::Element { attrs, .. } => attrs
            .borrow()
            .iter()
            .find(|attribute| &*attribute.name.local == key)
            .map(|attribute| attribute.value.to_string()),
        _ => None,
    }
}

pub fn set_attr(node: &Handle, key: &str, value: &str) {
    if let NodeData::Element { attrs, .. } = &node.data {
        if let Some(attribute) = attrs
            .borrow_mut()
            .iter_mut()
            .find(|attribute| &*attribute.name.local == key)
        {
            attribute.value = value.into();
        }
    }
}

pub fn remove_attr(node: &Handle, key: &str) {
    if let NodeData::Element { attrs, .. } = &node.data {
        attrs
            .borrow_mut()
            .retain(|attribute| &*attribute.name.local != key);
    }
}

pub fn has_class(node: &Handle, class: &str) -> bool {
    attr(node, "class").is_some_and(|value| value.split_whitespace().any(|name| name == class))
}

pub fn find(node: &Handle, matches: &dyn Fn(&Handle) -> bool) -> Option<Handle> {
    if matches(node) {
        return Some(node.clone());
    }
    node.children
        .borrow()
        .iter()
        .find_map(|child| find(child, matches))
}

pub fn find_all(node: &Handle, matches: &dyn Fn(&Handle) -> bool, found: &mut Vec<Handle>) {
    if matches(node) {
        found.push(node.clone());
        return;
    }
    for child in node.children.borrow().iter() {
        find_all(child, matches, found);
    }
}

pub fn each_element(node: &Handle, visit: &mut dyn FnMut(&Handle)) {
    if matches!(node.data, NodeData::Element { .. }) {
        visit(node);
    }
    for child in node.children.borrow().iter() {
        each_element(child, visit);
    }
}

pub fn text(node: &Handle) -> String {
    let mut out = String::new();
    collect_text(node, &mut out);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_text(node: &Handle, out: &mut String) {
    match &node.data {
        NodeData::Text { contents } => out.push_str(&contents.borrow()),
        NodeData::Element { name, .. } if matches!(&*name.local, "script" | "style") => {}
        _ => {
            for child in node.children.borrow().iter() {
                collect_text(child, out);
                if matches!(child.data, NodeData::Element { .. }) {
                    out.push(' ');
                }
            }
        }
    }
}
