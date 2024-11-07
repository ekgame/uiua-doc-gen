use std::fs::{create_dir_all, remove_dir};
use std::path::PathBuf;
use kuchiki::traits::TendrilSink;
use leptos::{view, CollectView, IntoView};
use leptos::html::Template;
use thiserror::Error;
use crate::extractor::ItemContent;
use crate::summarizer::{ContentItems, DocumentationSummary, RenderingContent, RenderingItem};

#[derive(Error, Debug)]
pub enum GenerationError {}

pub fn generate_documentation_site(directory: &PathBuf, summary: DocumentationSummary) -> Result<(), GenerationError> {
    let output_directory = directory.join("doc-site");
    remove_dir(output_directory.clone()).unwrap_or(());
    create_dir_all(output_directory.clone()).expect("Unable to create output directory");

    save_static_file(&output_directory, "style.css", include_bytes!("../design/style.css"));
    save_static_file(&output_directory, "script.js", include_bytes!("../design/script.js"));
    save_static_file(&output_directory, "Uiua386.ttf", include_bytes!("../design/Uiua386.ttf"));
    save_static_file(&output_directory, "index.html", generate_html(summary).as_bytes());

    Ok(())
}

fn save_static_file(output_directory: &PathBuf, file: &str, content: &[u8]) {
    let destination = output_directory.join(file);
    std::fs::write(destination, content).expect("Unable to write static file");
}

fn generate_html(summary: DocumentationSummary) -> String {
    let raw_output = leptos::ssr::render_to_string(|| generate_page(summary)).to_string();
    let document = kuchiki::parse_html().from_utf8().one(raw_output.as_bytes());

    // Remove comments
    document
        .inclusive_descendants()
        .filter(|node| node.as_comment().is_some())
        .for_each(|comment| {
            comment.detach()
        });

    // Remove data-hk attributes generated by leptos
    document
        .select("[data-hk]")
        .unwrap()
        .for_each(|node| {
            node.attributes.borrow_mut().remove("data-hk");
        });

    // Serialize back to string
    let mut result = Vec::new();
    document.serialize(&mut result).unwrap();
    String::from_utf8(result).unwrap()
}

fn generate_page(summary: DocumentationSummary) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <title>"Hello world"</title>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <link rel="stylesheet" href="style.css"/>
                <script src="script.js"></script>
            </head>
            <body>
                <div class="mobile-container">
                    <div class="mobile-nav">
                        <div class="hamburger">
                            <div class="line"></div>
                            <div class="line"></div>
                            <div class="line"></div>
                        </div>
                        <h1>{summary.title.clone()}</h1>
                    </div>
                    <div class="container">
                        <div class="sidebar">
                            {generate_sidebar(&summary)}
                        </div>
                        <div class="content">
                            <div class="content-wrapper">
                                <h1 class="mobile-hidden">{&summary.title}</h1>
                                {generate_content(&summary)}
                            </div>
                        </div>
                    </div>
                </div>
            </body>
        </html>
    }
}

fn generate_sidebar(summary: &DocumentationSummary) -> impl IntoView {
    view! {
        {summary.sections.iter()
            .map(|section| view! {
                <div class="sidebar-section">
                    <div class="section-name">{&section.title}</div>
                    <ul>
                        {section.content.iter()
                            .flat_map(|item| &item.links)
                            .map(|link| view! {
                                <li><a href={&link.url}>{&link.title}</a></li>
                            })
                            .collect_view()
                        }
                        {section.content.iter()
                            .filter(|item| matches!(&item.content, RenderingContent::Items(_)))
                            .map(|link| 
                                match &link.content {
                                    RenderingContent::Items(items) => view! {
                                        <li><a href={format!("#{}", items.title.link_id.clone())}>{items.title.title.clone()}</a></li>
                                    },
                                    _ => view! { <li>"N/A"</li> }
                                }
                            )
                            .collect_view()
                        }
                    </ul>
                </div>
            })
            .collect_view()
        }
    }
}

fn generate_content(summary: &DocumentationSummary) -> impl IntoView {
    view! {
        {summary.sections.iter()
            .map(|section| view! {
                {section.content.iter()
                    .map(|item| generate_rendering_item(item))
                    .collect_view()
                }
            })
            .collect_view()
        }
    }
}

fn generate_rendering_item(item: &RenderingItem) -> impl IntoView {
    match &item.content {
        RenderingContent::RenderedDocumentation(ref content) => view! {
            <div>
                <div class="panel" inner_html={content}></div>
            </div>
        },
        RenderingContent::Items(ref item) => view! {
            <div>
                <h2 id={&item.title.link_id}>{&item.title.title}</h2>
                {item.items.iter()
                    .map(|item| generate_content_item(item))
                    .collect_view()
                }
            </div>
        },
    }
}

fn generate_content_item(item: &ItemContent) -> impl IntoView {
    view! {
        <div>
            <div class="panel">"TODO"</div>
        </div>
    }
}