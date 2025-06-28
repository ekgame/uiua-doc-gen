use crate::extractor::{BindingDefinition, BindingType, FileContent, ItemContent, ModuleDefinition};
use crate::generator::markdown_to_html;
use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
use markup5ever::namespace_url;
use markup5ever::{local_name, ns, QualName};
use uiua::Compiler;
use std::option::Option;

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
pub enum SectionType {
    Documentation,
    Modules,
    Bindings,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct DocumentationSection {
    pub title: String,
    pub section_type: SectionType,
    pub content: Vec<RenderingItem>,
}

#[derive(Debug, Clone)]
pub struct DocumentationSummary {
    pub title: String,
    pub sections: Vec<DocumentationSection>,
}

pub fn summarize_content(content: &FileContent, title: String, compiler: &Compiler) -> DocumentationSummary {
    let mut sections = Vec::new();

    if let Some(documentation) = summarize_doc_comments(content, &compiler) {
        sections.push(documentation);
    }

    if let Some(modules) = summarize_modules(&content.items) {
        sections.push(DocumentationSection {
            title: "Modules".to_owned(),
            section_type: SectionType::Modules,
            content: modules
                .iter()
                .map(|item| {
                    if let ItemContent::Module(module) = item {
                        RenderingItem {
                            links: vec![],
                            content: RenderingContent::Items(ContentItems {
                                title: Title {
                                    title: module.name.clone(),
                                    link_id: module.name.clone(),
                                },
                                items: vec![item.clone()],
                            }),
                        }
                    } else {
                        panic!("Expected module item");
                    }
                })
                .collect(),
        });
    }

    if let Some(bindings) = summarize_bindings(&content.items) {
        sections.push(DocumentationSection {
            title: "Bindings".to_owned(),
            section_type: SectionType::Bindings,
            content: bindings,
        });
    }

    DocumentationSummary {
        title: title.clone(),
        sections,
    }
}

fn summarize_doc_comments(content: &FileContent, compiler: &Compiler) -> Option<DocumentationSection> {
    let doc_comments = extract_doc_comments(&content.items);
    if doc_comments.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    items.extend(doc_comments.iter().map(|comment| summarize_doc_comment(comment, &compiler)));

    if items.is_empty() {
        return None;
    }

    Some(DocumentationSection {
        title: "Documentation".to_owned(),
        section_type: SectionType::Documentation,
        content: items,
    })
}

fn summarize_doc_comment(comment: &str, compiler: &Compiler) -> RenderingItem {
    let mut links = Vec::new();

    let html = markdown_to_html(comment, &compiler);
    let document = kuchiki::parse_html().from_utf8().one(html.as_bytes());
    document
        .select("h1, h2, h3, h4, h5, h6")
        .unwrap()
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

            let new_header = NodeRef::new_element(QualName::new(None, ns!(html), new_level.clone()), None);

            new_header.append(NodeRef::new_text(element.text_contents()));

            if new_level.to_string() == "h2" {
                let title = element.text_contents();
                let id = title.to_lowercase().replace(' ', "-");
                new_header.as_element().unwrap().attributes.borrow_mut().insert("id", id.clone());
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
    let cleaned_comment = rendered_comment.replace("<html><head></head><body>", "").replace("</body></html>", "");

    RenderingItem {
        links,
        content: RenderingContent::RenderedDocumentation(cleaned_comment),
    }
}

fn extract_doc_comments(items: &[ItemContent]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| {
            if let ItemContent::Words { code } = item {
                if code.starts_with("# !doc") {
                    let comment = code
                        .lines()
                        .map(|line| line.trim_start_matches("# !doc").trim_start_matches('#').trim())
                        .collect::<Vec<&str>>()
                        .join("\n")
                        .trim()
                        .to_owned();
                    return Some(comment);
                }
            }
            None
        })
        .collect()
}

fn summarize_bindings(items: &[ItemContent]) -> Option<Vec<RenderingItem>> {
    let mut results = Vec::new();

    if let Some(item) = summarize_constants(items) {
        results.push(item);
    }

    if let Some(data) = summarize_data(items) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Data types".to_owned(),
                    link_id: "__data".to_owned(),
                },
                items: data,
            }),
        });
    }

    if let Some(macros) = summarize_code_macros(items) {
        results.push(RenderingItem {
            links: vec![],
            content: RenderingContent::Items(ContentItems {
                title: Title {
                    title: "Code macros".to_owned(),
                    link_id: "__code_macros".to_owned(),
                },
                items: macros,
            }),
        });
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

fn summarize_constants(items: &[ItemContent]) -> Option<RenderingItem> {
    let constants = items
        .iter()
        .filter(|item| {
            if let ItemContent::Binding(binding) = item {
                if let BindingType::Const(_) = &binding.kind {
                    return binding.public;
                }
            }
            false
        })
        .collect::<Vec<_>>();

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

fn summarize_functions(items: &[ItemContent], num_inputs: usize) -> Option<Vec<ItemContent>> {
    let functions = items
        .iter()
        .filter(|item| {
            if let ItemContent::Binding(binding) = item {
                if let BindingType::Function(function) = &binding.kind {
                    if function.signature().inputs == num_inputs {
                        return binding.public;
                    }
                }
            }
            false
        })
        .collect::<Vec<_>>();

    if functions.is_empty() {
        return None;
    }

    Some(functions.iter().map(|item| (*item).clone()).collect())
}

fn summarize_index_macros(items: &[ItemContent]) -> Option<Vec<ItemContent>> {
    let macros = items
        .iter()
        .filter(|item| {
            if let ItemContent::Binding(binding) = item {
                if let BindingType::IndexMacro(_) = &binding.kind {
                    return binding.public;
                }
            }
            false
        })
        .collect::<Vec<_>>();

    if macros.is_empty() {
        return None;
    }

    Some(macros.iter().map(|item| (*item).clone()).collect())
}

fn summarize_code_macros(items: &[ItemContent]) -> Option<Vec<ItemContent>> {
    let macros = items
        .iter()
        .filter(|item| {
            if let ItemContent::Binding(BindingDefinition {
                public,
                kind: BindingType::CodeMacro(_),
                ..
            }) = item
            {
                *public
            } else {
                false
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    if !macros.is_empty() {
        Some(macros)
    } else {
        None
    }
}

fn summarize_modules(items: &[ItemContent]) -> Option<Vec<ItemContent>> {
    let modules = items
        .iter()
        .filter(|item| {
            if let ItemContent::Module(module) = item {
                module.has_public_items()
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    if modules.is_empty() {
        return None;
    }

    Some(
        modules
            .iter()
            .map(|item| {
                ItemContent::Module(ModuleDefinition {
                    name: match item {
                        ItemContent::Module(module) => module.name.clone(),
                        _ => panic!("Expected module item"),
                    },
                    items: match item {
                        ItemContent::Module(module) => module
                            .items
                            .iter()
                            .filter(|item| match item {
                                ItemContent::Binding(binding) => binding.public,
                                ItemContent::Module(module) => module.has_public_items(),
                                ItemContent::Variant(_) => true,
                                ItemContent::Data(_) => true,
                                _ => false,
                            })
                            .cloned()
                            .collect(),
                        _ => panic!("Expected module item"),
                    },
                    comment: match item {
                        ItemContent::Module(module) => module.comment.clone(),
                        _ => None,
                    },
                })
            })
            .collect(),
    )
}

fn summarize_data(items: &[ItemContent]) -> Option<Vec<ItemContent>> {
    let data = items
        .iter()
        .filter(|item| matches!(item, ItemContent::Data(_) | ItemContent::Variant(_)))
        .cloned()
        .collect::<Vec<_>>();

    if !data.is_empty() {
        Some(data)
    } else {
        None
    }
}
