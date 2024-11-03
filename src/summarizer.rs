use html5ever::serialize::{HtmlSerializer, SerializeOpts, TraversalScope};
use html5ever::serialize::TraversalScope::IncludeNode;
use markup5ever::namespace_url;
use markup5ever::{local_name, ns, QualName};
use kuchiki::traits::{TendrilSink};
use kuchiki::{Attribute, ElementData, ExpandedName, NodeRef};
use uiua::LocalName;
use crate::extractor::{FileContent, ItemContent};

pub enum RenderingContent {
    RenderedDocumentation(String),
    Item(ItemContent),
}

pub struct ItemLink {
    pub title: String,
    pub url: String,
}

pub struct RenderingItem {
    pub links: Vec<ItemLink>,
    pub content: RenderingContent,
}

pub struct DocumentationSection {
    pub title: String,
    pub content: Vec<RenderingItem>,
}

pub struct DocumentationSummary {
    pub title: String,
    pub sections: Vec<DocumentationSection>,
}

pub fn summarize_content(content: &FileContent, title: String) -> DocumentationSummary {
    let mut sections = Vec::new();

    if let Some(documentation) = summarize_doc_comments(content) {
        sections.push(documentation);
    }

    DocumentationSummary {
        title: title.clone(),
        sections,
    }
}

fn summarize_doc_comments(content: &FileContent) -> Option<DocumentationSection> {
    let doc_comments = extract_doc_comments(&content.items);
    if doc_comments.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    items.extend(doc_comments.iter().map(summarize_doc_comment));

    if items.is_empty() {
        return None;
    }

    Some(DocumentationSection {
        title: "Documentation".to_owned(),
        content: items,
    })
}

fn summarize_doc_comment(comment: &String) -> RenderingItem {
    let mut links = Vec::new();

    let html = markdown::to_html_with_options(
        comment.as_str(),
        &markdown::Options::gfm()
    ).expect("Unable to convert markdown to HTML");

    let mut document = kuchiki::parse_html().from_utf8().one(html.as_bytes());
    document.select("h1, h2, h3, h4, h5, h6").unwrap()
        .for_each(|element| {
            // h1 -> h2, h2 -> h3, etc.
            let current_level = element.name.local.to_string();
            let new_level = match current_level.as_str() {
                "h1" => local_name!("h2"),
                "h2" => local_name!("h3"),
                "h3" => local_name!("h4"),
                "h4" => local_name!("h5"),
                _ => local_name!("h6"),
            };

            let mut new_header = NodeRef::new_element(
                QualName::new(None, ns!(html), new_level.clone()),
                None,
            );

            new_header.append(NodeRef::new_text(element.text_contents()));

            if new_header.as_element().unwrap().name.local.to_string() == "h2" {
                let title = element.text_contents();
                let id = title.to_lowercase().replace(" ", "-");
                new_header.as_element().unwrap().attributes.borrow_mut().insert("id", id.clone().into());
                links.push(ItemLink {
                    title,
                    url: format!("#{}", id),
                });
            }

            element.as_node().insert_after(new_header);
            element.as_node().detach();
        });

    // Serialize back to string
    let mut result = Vec::new();
    document.serialize(&mut result).unwrap();
    let rendered_comment = String::from_utf8(result).unwrap();
    let cleaned_comment = rendered_comment
        .replace("<html><head></head><body>", "")
        .replace("</body></html>", "");

    RenderingItem {
        links,
        content: RenderingContent::RenderedDocumentation(cleaned_comment),
    }
}

fn extract_doc_comments(items: &Vec<ItemContent>) -> Vec<String> {
    items.iter().filter_map(|item| {
        if let ItemContent::Words { code } = item {
            if code.starts_with("# !doc") {
                let comment = code.lines()
                    .map(|line| line.trim_start_matches("# !doc").trim_start_matches("#").trim())
                    .collect::<Vec<&str>>()
                    .join("\n")
                    .trim()
                    .to_owned();
                return Some(comment);
            }
        }
        None
    }).collect()
}