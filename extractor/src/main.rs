
use std::env;
use std::path::Path;
use uiua::ast::Item;
use uiua::Assembly;
use uiua::BindingInfo;
use uiua::CodeSpan;
use uiua::Compiler;
use uiua::NativeSys;
use uiua::{parse, InputSrc};

fn get_binding_info(asm: &Assembly, span: &CodeSpan) -> Option<BindingInfo> {
    for binding in &asm.bindings {
        if binding.span != *span {
            continue;
        }
        return Some(binding.clone());
    }
    panic!("Binding not found for span {:?}", span);
    None
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

    let mut comp = Compiler::with_backend(NativeSys::default());
    let asm = comp.load_file(lib_path).unwrap().finish();

    let mut inputs = asm.inputs.clone();
    let files: Vec<_> = inputs.files.iter()
        .map(|file| (file.key().clone(), file.value().clone())).collect();
    
    for (file_path, file_content) in files {
        let src = InputSrc::File(file_path.clone().into());
        let (items, errors, _) = parse(&file_content, src, &mut inputs);

        if errors.len() > 0 {
            eprintln!("Error: {} errors found in '{}'", errors.len(), file_path.to_str().unwrap());
            for error in errors {
                eprintln!("{}", error);
            }
            std::process::exit(1);
        }

        println!("");
        println!("Bindings in '{}':", file_path.to_str().unwrap());

        for item in items {
            match item {
                Item::Binding(binding) => {
                    println!("{:?}", binding.name);
                    let info = get_binding_info(&asm, &binding.name.span).unwrap();
                    println!("{:?}", info);
                }
                _ => {}
            }
        }
    }
}