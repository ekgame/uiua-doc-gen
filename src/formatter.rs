use leptos::view;
use leptos::*;
use uiua::{
    lsp::{BindingDocs, BindingDocsKind},
    Compiler, NativeSys, PrimClass, Primitive, Signature, SpanKind, Spans, Subscript,
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
enum CodeFragment {
    Unspanned(String),
    Br,
    Span(String, SpanKind),
}

struct CodeLines {
    frags: Vec<Vec<CodeFragment>>,
}

impl CodeLines {
    fn line(&mut self) -> &mut Vec<CodeFragment> {
        self.frags.last_mut().unwrap()
    }
    fn frag(&mut self) -> &mut CodeFragment {
        self.line().last_mut().unwrap()
    }
    fn push_str(&mut self, s: &str) {
        match self.frag() {
            CodeFragment::Unspanned(ref mut unspanned) => unspanned.push_str(s),
            _ => self.line().push(CodeFragment::Unspanned(s.to_string())),
        }
    }
    fn new_line(&mut self) {
        if self.line().is_empty() {
            self.line().push(CodeFragment::Br);
        }
        self.frags.push(Vec::new());
    }
}

fn modifier_class(margs: usize) -> &'static str {
    match margs {
        0 | 1 => "binding monadic-modifier",
        2 => "binding dyadic-modifier",
        _ => "binding triadic-modifier",
    }
}

fn sig_class(sig: Signature) -> &'static str {
    match sig.args() {
        0 => "binding noadic-function",
        1 => "binding monadic-function",
        2 => "binding dyadic-function",
        3 => "binding triadic-function",
        4 => "binding tetradic-function",
        _ => "binding",
    }
}

fn prim_sig_class(prim: Primitive, subscript: Option<Subscript>) -> &'static str {
    match prim {
        Primitive::Identity => "stack-function",
        prim if matches!(prim.class(), PrimClass::Stack | PrimClass::Debug) && prim.modifier_args().is_none() => "stack-function",
        prim if prim.class() == PrimClass::Constant => "number-literal",
        prim => {
            if let Some(m) = prim.modifier_args() {
                modifier_class(m)
            } else {
                prim.subscript_sig(subscript).or(prim.sig()).map(sig_class).unwrap_or("")
            }
        }
    }
}

fn binding_class(docs: &BindingDocs) -> &'static str {
    match docs.kind {
        BindingDocsKind::Constant(_) => "binding constant",
        BindingDocsKind::Function { sig, .. } => sig_class(sig),
        BindingDocsKind::Modifier(margs) => modifier_class(margs),
        BindingDocsKind::Module { .. } => "binding module",
        BindingDocsKind::Error => "output-error",
    }
}

fn build_code_lines(code: &str, compiler: &Compiler) -> CodeLines {
    let mut lines = CodeLines { frags: vec![Vec::new()] };

    let lib_file_src = &compiler.assembly().inputs.strings[0];
    let code_with_context = format!("{}\n\n{}", lib_file_src, &code);
    let chars: Vec<&str> = code_with_context.graphemes(true).collect();

    let push_unspanned = |lines: &mut CodeLines, mut target: usize, curr: &mut usize| {
        target = target.min(chars.len());
        if *curr >= target {
            return;
        }
        lines.line().push(CodeFragment::Unspanned(String::new()));
        let mut unspanned = String::new();
        while *curr < target {
            if chars[*curr] == "\n" {
                if !unspanned.is_empty() {
                    lines.push_str(&unspanned);
                    unspanned.clear();
                }
                lines.new_line();
                *curr += 1;
                while *curr < target && chars[*curr] == "\n" {
                    lines.new_line();
                    *curr += 1;
                }
                lines.line().push(CodeFragment::Unspanned(String::new()));
                continue;
            }
            unspanned.push_str(chars[*curr]);
            *curr += 1;
        }
        if !unspanned.is_empty() {
            lines.push_str(&unspanned);
        }
        lines.line().push(CodeFragment::Unspanned(String::new()));
    };

    let mut end = 0;

    let spans = Spans::with_backend(&code_with_context, NativeSys::default());
    for span in spans.spans {
        let kind = span.value;
        let span = span.span;
        push_unspanned(&mut lines, span.start.char_pos as usize, &mut end);

        let text: String = chars[span.start.char_pos as usize..span.end.char_pos as usize].iter().copied().collect();

        if !text.is_empty() && text.chars().all(|c| c == '\n') {
            lines.new_line();
            for _ in 0..text.chars().count() - 1 {
                lines.new_line();
            }
        } else {
            for (i, text) in text.lines().enumerate() {
                if i > 0 {
                    lines.new_line();
                }
                lines.line().push(CodeFragment::Span(text.into(), kind.clone()));
            }
        }

        end = span.end.char_pos as usize;
    }

    push_unspanned(&mut lines, chars.len(), &mut end);

    for line in &mut lines.frags {
        line.retain(|frag| !matches!(frag, CodeFragment::Unspanned(s) if s.is_empty()));
    }

    // count the lines in "code" and keep last N lines
    let code_lines_count = code.lines().count();
    if lines.frags.len() > code_lines_count {
        lines.frags = lines.frags[lines.frags.len() - code_lines_count..].to_vec();
    }

    lines
}

pub fn format_source_code(code: &str, compiler: &Compiler) -> String {
    let CodeLines { frags } = build_code_lines(code, &compiler);
    let mut line_views = Vec::new();
    for line in frags {
        if line.is_empty() {
            line_views.push(view! {
                <div class="code-line">
                    <br />
                </div>
            });
            continue;
        }
        let mut frag_views = Vec::new();
        let mut frags = line.into_iter().peekable();
        while let Some(frag) = frags.next() {
            match frag {
                CodeFragment::Unspanned(s) => frag_views.push(view! { <span class="code-span">{s}</span> }.into_view()),
                CodeFragment::Br => frag_views.push(view! { <br /> }.into_view()),
                CodeFragment::Span(text, kind) => {
                    let color_class: String = match &kind {
                        SpanKind::Primitive(prim, sig) => prim_sig_class(*prim, *sig).to_string(),
                        SpanKind::Obverse(_) => prim_sig_class(Primitive::Obverse, None).to_string(),
                        SpanKind::Number => "number-literal".to_string(),
                        SpanKind::String | SpanKind::ImportSrc(_) => "string-literal-span".to_string(),
                        SpanKind::Comment | SpanKind::OutputComment => "comment-span".to_string(),
                        SpanKind::Strand => "strand-span".to_string(),
                        SpanKind::Subscript(None, _) => "number-literal".to_string(),
                        SpanKind::Subscript(Some(prim), n) => prim_sig_class(*prim, *n).to_string(),
                        SpanKind::MacroDelim(margs) => modifier_class(*margs).to_string(),
                        SpanKind::ArgSetter(_) => sig_class((1, 0).into()).to_string(),
                        SpanKind::Ident { docs: Some(docs), .. } => binding_class(&docs).to_string(),
                        _ => "".to_string(),
                    };
                    let text = view! { <span class=format!("code-span {}", color_class)>{text}</span> };
                    frag_views.push(text.into_view());
                }
            }
        }

        line_views.push(view! { <div class="code-line">{frag_views}</div> })
    }

    ssr::render_to_string(|| line_views.into_view()).to_string()
}
