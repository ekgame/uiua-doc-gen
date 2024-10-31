extern crate uiua;

use std::fs::canonicalize;
use std::path::PathBuf;
use thiserror::Error;
use uiua::ast::Item;
use uiua::ast::ModuleKind;
use uiua::ast::Word;
use uiua::{Assembly, ParseError};
use uiua::BindingInfo;
use uiua::BindingKind;
use uiua::CodeSpan;
use uiua::Compiler;
use uiua::DocCommentSig;
use uiua::NativeSys;
use uiua::Signature;
use uiua::Sp;
use uiua::SysBackend;
use uiua::{parse, InputSrc};

#[derive(Debug)]
pub struct SignatureInfo {
    pub inputs: i32,
    pub outputs: i32,
}

#[derive(Debug)]
pub struct NamedSignature {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub validator: Option<String>,
}

#[derive(Debug)]
pub struct Definition {
    pub boxed: bool,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub enum ItemContent {
    Words { code: String },

    Binding {
        name: String,
        code: String,
        public: bool,
        comment: Option<String>,
        kind: BindingType,
    },

    Module {
        name: String,
        comment: Option<String>,
        items: Vec<ItemContent>,
    },

    Data {
        name: Option<String>,
        definition: Option<Definition>,
    },

    Variant {
        name: Option<String>,
        definition: Option<Definition>,
    },

    Import {
        path: String,
    },
}

#[derive(Debug)]
pub enum BindingType {
    Const { value: Option<String> },
    Function {
        signature: SignatureInfo,
        named_signature: Option<NamedSignature>,
    },
}

#[derive(Debug)]
pub struct FileContent {
    file: String,
    items: Vec<ItemContent>,
}

fn signature_comment_to_struct(doc: DocCommentSig) -> NamedSignature {
    let inputs = doc.args.iter()
        .map(|input| input.name.to_string())
        .collect();

    let outputs = doc.outputs
        .map(|outputs| outputs.iter()
            .map(|output| output.name.to_string())
            .collect())
        .unwrap_or_default();

    NamedSignature { inputs, outputs }
}

fn format_signature(signature: Signature) -> SignatureInfo {
    SignatureInfo {
        inputs: signature.args as i32,
        outputs: signature.outputs as i32,
    }
}

// Rest of the helper functions remain the same
fn get_binding_info(asm: &Assembly, span: &CodeSpan) -> Option<BindingInfo> {
    for binding in &asm.bindings {
        if binding.span != *span {
            continue;
        }
        return Some(binding.clone());
    }
    None
}

fn get_words_as_code_2(words: &Vec<Vec<Sp<Word>>>, asm: &Assembly) -> String {
    if words.first().unwrap().is_empty() {
        return "".to_string();
    }

    if words.last().unwrap().is_empty() {
        return "".to_string();
    }

    let from = &words.first().unwrap().first().unwrap().span;
    let to = &words.last().unwrap().last().unwrap().span;
    let span = from.clone().merge(to.clone());
    span.as_str(&asm.inputs, |code| code.to_owned())
}

fn get_words_as_code(words: &Vec<Sp<Word>>, asm: &Assembly) -> String {
    if words.is_empty() {
        return "".to_string();
    }

    let from = &words.first().unwrap().span;
    let to = &words.last().unwrap().span;
    let span = from.clone().merge(to.clone());
    span.as_str(&asm.inputs, |code| code.to_owned())
}

fn handle_ast_items(items: Vec<Item>, asm: &Assembly) -> Vec<ItemContent> {
    let mut results = Vec::new();

    for item in items {
        match item {
            Item::Words(words) => {
                let code_str = get_words_as_code_2(&words, asm).replace("\r\n", "\n");
                for chunk in code_str.split("\n\n") {
                    results.push(ItemContent::Words {
                        code: chunk.to_string(),
                    });
                }
            }
            Item::Binding(binding) => {
                let info = match get_binding_info(asm, &binding.name.span) {
                    Some(info) => info,
                    None => continue,
                };
                let code = binding.span().as_str(&asm.inputs, |code| code.to_owned());
                let comment = info.comment.clone().map(|comment| comment.text.to_string());
                let signature = info.comment.and_then(|comment| comment.sig);

                let kind = match info.kind {
                    BindingKind::Const(value) => BindingType::Const {
                        value: value.map(|v| v.to_string()),
                    },
                    BindingKind::Func(function) => BindingType::Function {
                        signature: format_signature(function.signature),
                        named_signature: signature.map(signature_comment_to_struct),
                    },
                    _ => continue,
                };

                results.push(ItemContent::Binding {
                    name: binding.name.value.to_string(),
                    code,
                    public: info.public,
                    comment,
                    kind,
                });
            }
            Item::Module(module) => {
                if let ModuleKind::Test = module.value.kind {
                    continue;
                }
                else if let ModuleKind::Named(name) = module.value.kind {
                    let info = match get_binding_info(asm, &name.span) {
                        Some(info) => info,
                        None => continue,
                    };

                    let comment = info.comment.map(|comment| comment.text.to_string());
                    let processed_items = handle_ast_items(module.value.items, asm);

                    results.push(ItemContent::Module {
                        name: name.value.to_string(),
                        comment,
                        items: processed_items,
                    });
                }
            }
            Item::Data(data_def) => {
                let definition = data_def.fields.map(|def| {
                    Definition {
                        boxed: def.boxed,
                        fields: def.fields.iter().map(|field| Field {
                            name: field.name.value.to_string(),
                            validator: field.validator.as_ref()
                                .map(|v| get_words_as_code(&v.words, asm)),
                        }).collect(),
                    }
                });

                let item_content = if data_def.variant {
                    ItemContent::Variant {
                        name: data_def.name.map(|name| name.value.to_string()),
                        definition,
                    }
                } else {
                    ItemContent::Data {
                        name: data_def.name.map(|name| name.value.to_string()),
                        definition,
                    }
                };

                results.push(item_content);
            }
            Item::Import(import) => {
                results.push(ItemContent::Import {
                    path: import.path.value.to_string(),
                });
            }
        }
    }

    results
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Library file not found: {0}")]
    LibraryNotFound(PathBuf),
    
    #[error("Failed to parse file: {0}")]
    ParseError(PathBuf, Sp<ParseError>),
}

pub fn extract_uiua_definitions(path: &PathBuf) -> Result<Vec<FileContent>, ExtractError> {
    let lib_path = path.join("lib.ua");
    if !lib_path.exists() || !lib_path.is_file() {
        return Err(ExtractError::LibraryNotFound(lib_path));
    }

    let backend = NativeSys::default();
    let _ = backend.change_directory(path.to_str().unwrap());

    let mut comp = Compiler::with_backend(backend);
    let asm = comp.load_file(lib_path).unwrap().finish();

    let mut inputs = asm.inputs.clone();
    let files: Vec<_> = inputs.files.iter()
        .map(|file| (file.key().clone(), file.value().clone())).collect();

    let mut output_files = Vec::new();

    for (file_path, file_content) in files {
        if file_path.starts_with("uiua-modules") {
            continue;
        }

        let full_file_path = canonicalize(&file_path).unwrap();
        let src = InputSrc::File(file_path.clone().into());
        let (items, errors, _) = parse(&file_content, src, &mut inputs);

        if !errors.is_empty() {
            return Err(ExtractError::ParseError(full_file_path, errors[0].clone()));
        }

        let file_content = FileContent {
            file: full_file_path.to_string_lossy().into_owned(),
            items: handle_ast_items(items, &asm),
        };

        output_files.push(file_content);
    }

    Ok(output_files)
}