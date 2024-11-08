use std::option::Option;
use markup5ever::namespace_url;
use markup5ever::{local_name, ns, QualName};
use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
use crate::extractor::{BindingType, FileContent, ItemContent};

#[derive(Debug, Clone)]
pub struct Title {
    pub title: String,
    pub link_id: String,
}

#[derive(Debug, Clone)]
pub struct ContentItems {
    pub title: Title,
    pub items: Vec<ItemContent>,
}

#[derive(Debug, Clone)]
pub enum RenderingContent {
    RenderedDocumentation(String),
    Items(ContentItems),
}

#[derive(Debug, Clone)]
pub struct ItemLink {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct RenderingItem {
    pub links: Vec<ItemLink>,
    pub content: RenderingContent,
}

#[derive(Debug, Clone)]
pub struct DocumentationSection {
    pub title: String,
    pub content: Vec<RenderingItem>,
}

#[derive(Debug, Clone)]
pub struct DocumentationSummary {
    pub title: String,
    pub sections: Vec<DocumentationSection>,
}

pub fn summarize_content(content: &FileContent, title: String) -> DocumentationSummary {
    let mut sections = Vec::new();

    if let Some(documentation) = summarize_doc_comments(content) {
        sections.push(documentation);
    }
    
    if let Some(bindings) = summarize_bindings(&content.items) {
        sections.push(DocumentationSection {
            title: "Bindings".to_owned(),
            content: bindings,
        });
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

    let document = kuchiki::parse_html().from_utf8().one(html.as_bytes());
    document.select("h1, h2, h3, h4, h5, h6").unwrap()
        .collect::<Vec<_>>()
        .into_iter()
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

            let new_header = NodeRef::new_element(
                QualName::new(None, ns!(html), new_level.clone()),
                None,
            );

            new_header.append(NodeRef::new_text(element.text_contents()));

            if new_level.to_string() == "h2" {
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

fn summarize_bindings(items: &Vec<ItemContent>) -> Option<Vec<RenderingItem>> {
    let mut results = Vec::new();
    
    if let Some(item) = summarize_constants(items) {
        results.push(item);
    }
    
    if let Some(macros) = summarize_index_macros(items) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Index macros".to_owned(),
                    link_id: "__index_macros".to_owned(),
                },
                items: macros,
            }),
        });
    }
    
    if let Some(functions) = summarize_functions(items, 0) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Noadic functions".to_owned(),
                    link_id: "__noadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 1) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Monadic functions".to_owned(),
                    link_id: "__monadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 2) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Dyadic functions".to_owned(),
                    link_id: "__dyadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 3) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Triadic functions".to_owned(),
                    link_id: "__triadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 4) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Tetradic functions".to_owned(),
                    link_id: "__tetradic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 5) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Pentadic functions".to_owned(),
                    link_id: "__pentadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }

    if let Some(functions) = summarize_functions(items, 6) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Hexadic functions".to_owned(),
                    link_id: "__hexadic_functions".to_owned(),
                },
                items: functions,
            }),
        });
    }
    
    Some(results)
}

fn summarize_constants(items: &Vec<ItemContent>) -> Option<RenderingItem> {
    let constants = items.iter().filter(|item| {
        if let ItemContent::Binding(binding) = item {
            if let BindingType::Const(_) = &binding.kind {
                return true;
            }
        }
        false
    }).collect::<Vec<_>>();
    
    if constants.is_empty() {
        return None;
    }
    
    Some(RenderingItem {
        links: vec![],
        content: RenderingContent::Items(ContentItems {
            title: Title {
                title: "Constants".to_owned(),
                link_id: "__constants".to_owned(),
            },
            items: constants.iter().map(|item| (*item).clone()).collect(),
        }),
    })
}

fn summarize_functions(items: &Vec<ItemContent>, num_inputs: i32) -> Option<Vec<ItemContent>> {
    let functions = items.iter().filter(|item| {
        if let ItemContent::Binding(binding) = item {
            if let BindingType::Function(function) = &binding.kind {
                if function.signature.inputs == num_inputs {
                    return true;
                }
            }
        }
        false
    }).collect::<Vec<_>>();
    
    if functions.is_empty() {
        return None;
    }
    
    Some(functions.iter().map(|item| (*item).clone()).collect())
}

fn summarize_index_macros(items: &Vec<ItemContent>) -> Option<Vec<ItemContent>> {
    let macros = items.iter().filter(|item| {
        if let ItemContent::Binding(binding) = item {
            if let BindingType::IndexMacro(_) = &binding.kind {
                return true;
            }
        }
        false
    }).collect::<Vec<_>>();
    
    if macros.is_empty() {
        return None;
    }
    
    Some(macros.iter().map(|item| (*item).clone()).collect())
}