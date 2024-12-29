extern crate uiua;

use same_file::is_same_file;
use std::fmt;
use std::fs::canonicalize;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

use uiua::{
    ast::{Item, ModuleKind, Word},
    parse, Assembly, BindingInfo, BindingKind, CodeSpan, Compiler, DocCommentSig, InputSrc, NativeSys, ParseError, Signature, Sp, SysBackend,
};

#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub inputs: usize,
    pub outputs: usize,
}

impl fmt::Display for SignatureInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.outputs == 1 {
            write!(f, "|{}", self.inputs)
        } else {
            write!(f, "|{}.{}", self.inputs, self.outputs)
        }
    }
}

impl Colored for SignatureInfo {
    fn color_class(&self) -> &'static str {
        match self.inputs {
            0 => "noadic-function",
            1 => "monadic-function",
            2 => "dyadic-function",
            3 => "triadic-function",
            4 => "tetradic-function",
            _ => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedSignature {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub validator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub boxed: bool,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct BindingDefinition {
    pub name: String,
    pub code: String,
    pub public: bool,
    pub comment: Option<String>,
    pub kind: BindingType,
}

impl Documented for BindingDefinition {
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub name: String,
    pub comment: Option<String>,
    pub items: Vec<ItemContent>,
}

impl Documented for ModuleDefinition {
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

impl ModuleDefinition {
    pub fn has_public_items(&self) -> bool {
        self.items.iter().any(|item| match item {
            ItemContent::Binding(binding) => binding.public,
            ItemContent::Module(module) => module.has_public_items(),
            ItemContent::Data(_) => true,
            ItemContent::Variant(_) => true,
            _ => false,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataDefinition {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub definition: Option<Definition>,
}

impl Documented for DataDefinition {
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct VariantDefinition {
    pub name: String,
    pub comment: Option<String>,
    pub definition: Option<Definition>,
}

impl Documented for VariantDefinition {
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ImportDefinition {
    path: String,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum ItemContent {
    Words { code: String },

    Binding(BindingDefinition),
    Module(ModuleDefinition),
    Data(DataDefinition),
    Variant(VariantDefinition),
    Import(ImportDefinition),
}

#[derive(Debug, Clone)]
pub struct ConstantDefinition {
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub signature: SignatureInfo,
    pub named_signature: Option<NamedSignature>,
}

#[derive(Debug, Clone)]
pub struct IndexMacroDefinition {
    pub arguments: usize,
    pub named_signature: Option<NamedSignature>,
}

impl Colored for IndexMacroDefinition {
    fn color_class(&self) -> &'static str {
        match self.arguments {
            1 => "monadic-modifier",
            2 => "dyadic-modifier",
            _ => "triadic-modifier",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeMacroDefinition {
    pub named_signature: Option<NamedSignature>,
}

#[derive(Debug, Clone)]
pub enum BindingType {
    Const(ConstantDefinition),
    Function(FunctionDefinition),
    IndexMacro(IndexMacroDefinition),
    CodeMacro(CodeMacroDefinition),
}

#[derive(Debug)]
#[allow(unused)]
pub struct FileContent {
    pub main: bool,
    pub file: String,
    pub items: Vec<ItemContent>,
}

pub trait Colored {
    fn color_class(&self) -> &'static str;
}

pub trait Documented {
    fn comment(&self) -> Option<&str>;
}

impl From<DocCommentSig> for NamedSignature {
    fn from(doc: DocCommentSig) -> Self {
        let inputs = doc
            .args
            .map(|inputs| inputs.iter().map(|output| output.name.to_string()).collect())
            .unwrap_or_default();
        
        let outputs = doc
            .outputs
            .map(|outputs| outputs.iter().map(|output| output.name.to_string()).collect())
            .unwrap_or_default();

        NamedSignature { inputs, outputs }
    }
}

impl From<Signature> for SignatureInfo {
    fn from(sig: Signature) -> Self {
        SignatureInfo {
            inputs: sig.args,
            outputs: sig.outputs,
        }
    }
}

// Rest of the helper functions remain the same
fn get_binding_info(asm: &Assembly, span: &CodeSpan) -> Option<BindingInfo> {
    asm.bindings.iter().find(|binding| binding.span == *span).cloned()
}

fn get_words_as_code_2(words: &[Vec<Sp<Word>>], asm: &Assembly) -> String {
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

fn get_words_as_code(words: &[Sp<Word>], asm: &Assembly) -> String {
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
                    results.push(ItemContent::Words { code: chunk.to_string() });
                }
            }
            Item::Binding(binding) => {
                let info = match get_binding_info(asm, &binding.name.span) {
                    Some(info) => info,
                    None => continue,
                };
                let code = binding.span().as_str(&asm.inputs, |code| code.to_owned());
                let comment = info.meta.comment.clone().map(|comment| comment.text.to_string());
                let signature = info.meta.comment.and_then(|comment| comment.sig);

                let kind = match info.kind {
                    BindingKind::Const(value) => BindingType::Const(ConstantDefinition {
                        value: value.map(|v| v.show()),
                    }),
                    BindingKind::Func(function) => BindingType::Function(FunctionDefinition {
                        signature: function.sig.into(),
                        named_signature: signature.map(Into::into),
                    }),
                    BindingKind::IndexMacro(code_macro_args) => BindingType::IndexMacro(IndexMacroDefinition {
                        arguments: code_macro_args,
                        named_signature: signature.map(Into::into),
                    }),
                    BindingKind::CodeMacro(_) => BindingType::CodeMacro(CodeMacroDefinition {
                        named_signature: signature.map(Into::into),
                    }),
                    _ => continue,
                };

                results.push(ItemContent::Binding(BindingDefinition {
                    name: binding.name.value.to_string(),
                    code,
                    public: info.public,
                    comment,
                    kind,
                }));
            }
            Item::Module(module) => {
                if let ModuleKind::Test = module.value.kind {
                    continue;
                } else if let ModuleKind::Named(name) = module.value.kind {
                    let info = match get_binding_info(asm, &name.span) {
                        Some(info) => info,
                        None => continue,
                    };

                    let comment = info.meta.comment.map(|comment| comment.text.to_string());
                    let processed_items = handle_ast_items(module.value.items, asm);

                    results.push(ItemContent::Module(ModuleDefinition {
                        name: name.value.to_string(),
                        comment,
                        items: processed_items,
                    }));
                }
            }
            Item::Data(data_def) => {
                let definition = data_def.fields.map(|def| Definition {
                    boxed: def.boxed,
                    fields: def
                        .fields
                        .iter()
                        .map(|field| Field {
                            name: field.name.value.to_string(),
                            validator: field.validator.as_ref().map(|v| get_words_as_code(&v.words, asm)),
                        })
                        .collect(),
                });

                let info = match &data_def.name {
                    Some(name) => match get_binding_info(asm, &name.span) {
                        Some(info) => Some(info),
                        None => panic!("Data definition without binding info"),
                    },
                    None => None,
                };

                let item_content = if data_def.variant {
                    ItemContent::Variant(VariantDefinition {
                        name: data_def.name.map(|name| name.value.to_string()).unwrap(),
                        comment: info.and_then(|info| info.meta.comment.map(|comment| comment.text.to_string())),
                        definition,
                    })
                } else {
                    ItemContent::Data(DataDefinition {
                        name: data_def.name.map(|name| name.value.to_string()),
                        comment: info.and_then(|info| info.meta.comment.map(|comment| comment.text.to_string())),
                        definition,
                    })
                };

                results.push(item_content);
            }
            Item::Import(import) => {
                results.push(ItemContent::Import(ImportDefinition {
                    path: import.path.value.to_string(),
                }));
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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Uiua Error: {0}")]
    UiuaError(#[from] uiua::UiuaError),
}

pub fn extract_uiua_definitions(path: &Path) -> Result<Vec<FileContent>, ExtractError> {
    let lib_path = path.join("lib.ua");
    if !lib_path.exists() || !lib_path.is_file() {
        return Err(ExtractError::LibraryNotFound(lib_path));
    }

    let backend = NativeSys;
    let _ = backend.change_directory(path.to_str().unwrap());

    let mut comp = Compiler::with_backend(backend);
    let asm = comp.load_file(&lib_path)?.finish();

    let mut inputs = asm.inputs.clone();
    let files: Vec<_> = inputs.files.iter().map(|file| (file.key().clone(), file.value().clone())).collect();

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
            main: is_same_file(&full_file_path, &lib_path)?,
            file: full_file_path.to_string_lossy().into_owned(),
            items: handle_ast_items(items, &asm),
        };

        output_files.push(file_content);
    }

    Ok(output_files)
}
