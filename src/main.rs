
use std::env;
use std::fs::canonicalize;
use std::path::Path;
use serde_json::Map;
use serde_json::Value;
use uiua::ast::Item;
use uiua::ast::ModuleKind;
use uiua::ast::Word;
use uiua::Assembly;
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

fn get_binding_info(asm: &Assembly, span: &CodeSpan) -> Option<BindingInfo> {
    for binding in &asm.bindings {
        if binding.span != *span {
            continue;
        }
        return Some(binding.clone());
    }

    None
}

fn signature_comment_to_object(doc: DocCommentSig) -> Value {
    let mut inputs = Vec::new();
    doc.args.iter().for_each(|input|
        inputs.push(input.name.to_string())
    );

    let mut outputs = Vec::new();
    doc.outputs.and_then(|output|
        Some(output.iter().for_each(|output|
            outputs.push(output.name.to_string())
        ))
    );

    let mut output = Map::new();
    output.insert("outputs".to_string(), Value::Array(outputs.into_iter().map(Value::String).collect()));
    output.insert("inputs".to_string(), Value::Array(inputs.into_iter().map(Value::String).collect()));
    
    Value::Object(output)
}

fn format_signature(signature: Signature) -> Value {
    let mut output = Map::new();
    output.insert("inputs".to_string(), Value::Number(signature.args.into()));
    output.insert("outputs".to_string(), Value::Number(signature.outputs.into()));
    Value::Object(output)
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

fn handle_ast_items(items: Vec<Item>, asm: &Assembly) -> Vec<Value> {
    let mut results = Vec::new();

    for item in items {
        match item {
            Item::Words(words) => {
                let code_str = get_words_as_code_2(&words, asm).replace("\r\n", "\n");
                let code = code_str.split("\n\n");

                for chunk in code {
                    let mut output = Map::new();
                    output.insert("type".to_string(), Value::String("words".to_string()));
                    output.insert("code".to_string(), Value::String(chunk.to_string()));
                    results.push(Value::Object(output));
                }
            }
            Item::Binding(binding) => {
                let mut output = Map::new();

                let info = match get_binding_info(asm, &binding.name.span) {
                    Some(info) => info,
                    None => continue,
                };
                let code = binding.span().as_str(&asm.inputs, |code| code.to_owned());
                let comment = info.comment.clone().map_or(Value::Null, |comment| Value::String(comment.text.to_string()));
                let signature = info.comment.and_then(|comment| comment.sig);
                
                output.insert("type".to_string(), Value::String("binding".to_string()));
                output.insert("name".to_string(), Value::String(binding.name.value.to_string()));
                output.insert("code".to_string(), Value::String(code));
                output.insert("public".to_string(), Value::Bool(info.public));
                output.insert("comment".to_string(), comment);

                match info.kind {
                    BindingKind::Const(value) => {
                        output.insert("kind".to_string(), Value::String("const".to_string()));
                        output.insert("value".to_string(), value.map_or(Value::Null, |v| Value::String(v.to_string())));
                    },
                    BindingKind::Func(function) => {
                        output.insert("kind".to_string(), Value::String("func".to_string()));
                        output.insert("signature".to_string(), format_signature(function.signature));
                        output.insert("named_signature".to_string(), signature.map_or(Value::Null, signature_comment_to_object));
                    }
                    _ => {}
                }

                results.push(Value::Object(output));
            }
            Item::Module(module) => {
                if let ModuleKind::Test = module.value.kind {
                    // We do not document tests
                    continue
                }
                else if let ModuleKind::Named(name) = module.value.kind {
                    let info = match get_binding_info(asm, &name.span) {
                        Some(info) => info,
                        None => continue,
                    };

                    let comment = info.comment.clone().map_or(Value::Null, |comment| Value::String(comment.text.to_string()));

                    let mut output = Map::new();
                    output.insert("type".to_string(), Value::String("module".to_string()));
                    output.insert("name".to_string(), Value::String(name.value.to_string()));
                    output.insert("comment".to_string(), comment);
                    
                    let processed_items = handle_ast_items(module.value.items, asm);
                    output.insert("items".to_string(), Value::Array(processed_items));

                    results.push(Value::Object(output));
                }
            }
            Item::Data(data_def) => {
                let mut output = Map::new();

                let data_def_name = data_def.name.clone();
                output.insert("name".to_string(), data_def_name.map_or(Value::Null, |name| Value::String(name.value.to_string())));


                if data_def.variant {
                    output.insert("type".to_string(), Value::String("variant".to_string()));
                } else {
                    output.insert("type".to_string(), Value::String("data".to_string()));
                }
                
                if let Some(def) = data_def.fields {
                    let fields: Vec<Value> = def.fields.iter().map(|field| {
                        let mut field_obj = Map::new();
                        field_obj.insert("name".to_string(), Value::String(field.name.value.to_string()));
                        field_obj.insert("validator".to_string(), field.validator.as_ref().map_or(Value::Null, |v| Value::String(get_words_as_code(&v.words, asm))));
                        Value::Object(field_obj)
                    }).collect();

                    let mut definition = Map::new();
                    definition.insert("boxed".to_string(), Value::Bool(def.boxed));
                    definition.insert("fields".to_string(), Value::Array(fields));
                    output.insert("definition".to_string(), Value::Object(definition));
                } else {
                    output.insert("definition".to_string(), Value::Null);
                }

                results.push(Value::Object(output));
            }
            Item::Import(import) => {
                let mut output = Map::new();
                output.insert("type".to_string(), Value::String("import".to_string()));
                output.insert("path".to_string(), Value::String(import.path.value.to_string()));
                results.push(Value::Object(output));
            }
        }
    }

    results
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        std::process::exit(1);
    }

    let dir_path = &args[1];
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        eprintln!("Error: '{}' is not a valid directory", dir_path);
        std::process::exit(1);
    }

    let lib_path = path.join("lib.ua");
    if !lib_path.exists() || !lib_path.is_file() {
        eprintln!("Error: File '{}' does not exist", lib_path.display());
        std::process::exit(1);
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
        let mut output_file = Map::new();
        if file_path.starts_with("uiua-modules") {
            continue;
        }

        let full_file_path = canonicalize(&file_path).unwrap();
        let src = InputSrc::File(file_path.clone().into());
        let (items, errors, _) = parse(&file_content, src, &mut inputs);

        output_file.insert("file".to_string(), serde_json::Value::String(full_file_path.to_string_lossy().into_owned()));

        if errors.len() > 0 {
            eprintln!("Error: {} errors found in '{}'", errors.len(), file_path.to_str().unwrap());
            for error in errors {
                eprintln!("{}", error);
            }
            std::process::exit(1);
        }

        let processed_items = handle_ast_items(items, &asm);
        output_file.insert("items".to_string(), serde_json::Value::Array(processed_items));
        output_files.push(Value::Object(output_file));
    }

    let output = Value::Array(output_files);
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}