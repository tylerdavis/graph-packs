use crate::dom;
use html5ever::{local_name, ns, QualName};
use markup5ever_rcdom::{Handle, Node, NodeData};
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

const ALWAYS_SKIPPED: &[&str] = &[
    "head", "script", "style", "noscript", "template", "svg", "iframe", "button", "nav", "aside",
];

const SKIPPED_OUTSIDE_MAIN: &[&str] = &["header", "footer"];

const INLINE: &[&str] = &[
    "a", "abbr", "b", "cite", "code", "em", "i", "kbd", "label", "mark", "q", "s", "samp", "small",
    "span", "strong", "sub", "sup", "time", "u", "var",
];

const LINE_BREAKING: &[&str] = &["div", "p", "li", "tr", "br", "section", "table"];

#[derive(Debug, PartialEq)]
pub struct Page {
    pub title: Option<String>,
    pub markdown: String,
}

pub fn convert(html: &str, base: Option<&Url>) -> Page {
    let document = dom::parse(html);
    let title = dom::find(&document, &|node| {
        dom::tag(node).as_deref() == Some("title")
    })
    .map(|node| dom::text(&node))
    .filter(|title| !title.is_empty())
    .or_else(|| {
        dom::find(&document, &|node| dom::tag(node).as_deref() == Some("h1"))
            .map(|node| dom::text(&node))
            .filter(|title| !title.is_empty())
    });
    if let Some(base) = base {
        absolutize(&document, base);
    }
    flatten_code(&document);
    space_inline_siblings(&document);
    let main = content_root(&document);
    let mut skipped: Vec<&str> = ALWAYS_SKIPPED.to_vec();
    if main.is_none() {
        skipped.extend(SKIPPED_OUTSIDE_MAIN);
    }
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(skipped)
        .options(htmd::options::Options {
            heading_style: htmd::options::HeadingStyle::Atx,
            code_block_style: htmd::options::CodeBlockStyle::Fenced,
            ..Default::default()
        })
        .build();
    let roots = main.unwrap_or_else(|| vec![document.clone()]);
    let markdown = roots
        .iter()
        .map(|root| converter.tree_to_markdown(root))
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    Page {
        title,
        markdown: tidy(&markdown),
    }
}

fn content_root(document: &Handle) -> Option<Vec<Handle>> {
    for tag in ["main", "article"] {
        let mut found = Vec::new();
        dom::find_all(
            document,
            &|node| dom::tag(node).as_deref() == Some(tag),
            &mut found,
        );
        let found: Vec<Handle> = found
            .into_iter()
            .filter(|node| !dom::text(node).is_empty())
            .collect();
        if !found.is_empty() {
            return Some(found);
        }
    }
    let mut role_main = Vec::new();
    dom::find_all(
        document,
        &|node| dom::attr(node, "role").as_deref() == Some("main"),
        &mut role_main,
    );
    (!role_main.is_empty()).then_some(role_main)
}

fn absolutize(document: &Handle, base: &Url) {
    dom::each_element(document, &mut |node| {
        let key = match dom::tag(node).as_deref() {
            Some("a") => "href",
            Some("img") => "src",
            _ => return,
        };
        let Some(value) = dom::attr(node, key) else {
            return;
        };
        if key == "href"
            && dom::text(node).is_empty()
            && dom::find(node, &|inner| dom::tag(inner).as_deref() == Some("img")).is_none()
        {
            dom::remove_attr(node, key);
            return;
        }
        if value.starts_with("javascript:") || value.starts_with("data:") {
            return;
        }
        if let Ok(joined) = base.join(&value) {
            dom::set_attr(node, key, joined.as_str());
        }
    });
}

fn text_node(text: &str, parent: &Handle) -> Handle {
    let node = Node::new(NodeData::Text {
        contents: RefCell::new(text.into()),
    });
    node.parent.set(Some(Rc::downgrade(parent)));
    node
}

fn code_element() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("code")),
        attrs: RefCell::new(Vec::new()),
        template_contents: RefCell::new(None),
        mathml_annotation_xml_integration_point: false,
    })
}

fn flatten_code(document: &Handle) {
    let mut blocks = Vec::new();
    dom::find_all(
        document,
        &|node| dom::tag(node).as_deref() == Some("pre"),
        &mut blocks,
    );
    for pre in blocks {
        let mut code = String::new();
        raw_text(&pre, &mut code);
        let code = code.trim_matches('\n').to_string();
        let holder = pre
            .children
            .borrow()
            .iter()
            .find_map(|child| dom::find(child, &|node| dom::tag(node).as_deref() == Some("code")))
            .unwrap_or_else(code_element);
        *pre.children.borrow_mut() = vec![holder.clone()];
        holder.parent.set(Some(Rc::downgrade(&pre)));
        *holder.children.borrow_mut() = vec![text_node(&code, &holder)];
    }
}

fn raw_text(node: &Handle, out: &mut String) {
    for child in node.children.borrow().iter() {
        match &child.data {
            NodeData::Text { contents } => out.push_str(&contents.borrow()),
            NodeData::Element { .. } => {
                let tag = dom::tag(child).unwrap_or_default();
                if tag == "br" {
                    out.push('\n');
                    continue;
                }
                if ALWAYS_SKIPPED.contains(&tag.as_str()) {
                    continue;
                }
                raw_text(child, out);
                if LINE_BREAKING.contains(&tag.as_str()) && !out.ends_with('\n') {
                    out.push('\n');
                }
            }
            _ => {}
        }
    }
}

fn space_inline_siblings(node: &Handle) {
    if dom::tag(node).as_deref() == Some("pre") {
        return;
    }
    let inline = |child: &Handle| dom::tag(child).is_some_and(|tag| INLINE.contains(&tag.as_str()));
    let children: Vec<Handle> = node.children.borrow().clone();
    let mut spaced = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        if index > 0 && inline(&children[index - 1]) && inline(child) {
            spaced.push(text_node(" ", node));
        }
        spaced.push(child.clone());
    }
    if spaced.len() != children.len() {
        *node.children.borrow_mut() = spaced;
    }
    for child in children {
        space_inline_siblings(&child);
    }
}

fn tidy(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut blank = 0;
    let mut fenced = false;
    for line in markdown.lines() {
        let line = line.trim_end();
        if line.starts_with("```") {
            fenced = !fenced;
        }
        let unescaped;
        let line = if fenced {
            line
        } else {
            unescaped = line.replace("\\_", "_");
            unescaped.as_str()
        };
        if line.is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

pub fn truncate(text: &str, max_chars: usize) -> (String, bool) {
    match text.char_indices().nth(max_chars) {
        None => (text.to_string(), false),
        Some((cut, _)) => {
            let head = &text[..cut];
            let at = head
                .rfind("\n\n")
                .or_else(|| head.rfind('\n'))
                .filter(|at| *at >= cut / 2)
                .unwrap_or(cut);
            (head[..at].trim_end().to_string(), true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCS: &str = include_str!("../fixtures/docs_page.html");

    #[test]
    fn narrows_to_main_and_keeps_headings_code_tables_and_parameters() {
        let base = Url::parse("https://docs.example.com/api/customers/create").unwrap();
        let page = convert(DOCS, Some(&base));
        assert_eq!(
            page.title.as_deref(),
            Some("Create a customer | Example API")
        );
        let md = &page.markdown;
        assert!(md.contains("# Create a customer"), "{md}");
        assert!(md.contains("## Parameters\n"), "{md}");
        assert!(!md.contains("[]("), "{md}");
        for name in ["description", "email", "name", "phone"] {
            assert!(md.contains(&format!("`{name}`")), "{name} missing:\n{md}");
        }
        assert!(md.contains("### payment_method string"), "{md}");
        assert!(
            md.contains("```\nfont-family = \"Example Mono\"\n```"),
            "{md}"
        );
        assert!(
            md.contains("```\n{\n  \"id\": \"cus_123\",\n  \"object\": \"customer\"\n}\n```"),
            "{md}"
        );
        assert!(
            md.contains("curl https://api.example.com/v1/customers"),
            "{md}"
        );
        assert!(
            md.contains("| Code |") && md.contains("| 401  | Unauthorized |"),
            "{md}"
        );
        assert!(
            md.contains("(https://docs.example.com/api/customers/update)"),
            "{md}"
        );
        for dropped in [
            "Site navigation",
            "Pricing",
            "On this page",
            "trackPageView",
            "Copy",
            "© Example",
        ] {
            assert!(!md.contains(dropped), "{dropped} kept:\n{md}");
        }
    }

    #[test]
    fn without_main_or_article_converts_the_body_and_drops_site_chrome() {
        let html = "<html><head><title>Options</title><style>.x{}</style></head><body>\
            <header><a href=\"/\">Home</a></header>\
            <nav><ul><li>Docs</li></ul></nav>\
            <h2 id=\"font-family\">font-family</h2><p>The font to use.</p>\
            <h2 id=\"font-size\">font-size</h2><p>Points.</p>\
            <footer>Footer text</footer>\
            <script>var x = 1;</script></body></html>";
        let page = convert(html, None);
        assert_eq!(page.title.as_deref(), Some("Options"));
        assert!(
            page.markdown.contains("## font-family"),
            "{}",
            page.markdown
        );
        assert!(page.markdown.contains("## font-size"), "{}", page.markdown);
        for dropped in ["Home", "Docs", "Footer text", "var x"] {
            assert!(
                !page.markdown.contains(dropped),
                "{dropped}: {}",
                page.markdown
            );
        }
    }

    #[test]
    fn an_article_is_used_when_there_is_no_main() {
        let html = "<body><div>Sidebar junk</div><article><header><h1>Post</h1></header><p>Body.</p></article></body>";
        let page = convert(html, None);
        assert_eq!(page.markdown, "# Post\n\nBody.");
        assert_eq!(page.title.as_deref(), Some("Post"));
    }

    #[test]
    fn truncation_cuts_at_a_paragraph_and_reports_it() {
        let text = format!("{}\n\n{}", "a".repeat(60), "b".repeat(60));
        assert_eq!(truncate(&text, 1000), (text.clone(), false));
        let (cut, truncated) = truncate(&text, 100);
        assert!(truncated);
        assert_eq!(cut, "a".repeat(60));
        let (cut, truncated) = truncate("ééééé", 3);
        assert!(truncated);
        assert_eq!(cut, "ééé");
    }
}
