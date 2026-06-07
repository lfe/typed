mod eetf;
mod error;
mod lower;
mod sexp;
mod typed_surface;

use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: typed-check <file.lfe> [--output <file.eetf>]");
        process::exit(2);
    }

    let input_file = &args[1];
    let output_file = args.iter().position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let source_name = Path::new(input_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(input_file);

    let forms = match sexp::Parser::parse_all_file(input_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {}", input_file, e);
            process::exit(1);
        }
    };

    let mut module_name = String::new();
    let mut module_exports = Vec::new();
    let mut lowered_funs = Vec::new();
    let mut had_error = false;

    for form in &forms {
        if let Some(mdef) = typed_surface::extract_module_def(form) {
            module_name = mdef.name;
            module_exports = mdef.exports;
            continue;
        }

        match &form {
            sexp::types::SExp::List(l)
                if !l.elements.is_empty()
                    && matches!(&l.elements[0], sexp::types::SExp::Symbol(s) if s.value == "defun/typed") =>
            {
                match typed_surface::extract_typed_fun(form) {
                    Ok(tf) => {
                        let lowered = lower::lower_typed_fun(&tf);
                        lowered_funs.push(lowered);
                    }
                    Err(e) => {
                        let e = match e {
                            error::CheckError::Diagnostic { pos, message, .. } => {
                                error::CheckError::Diagnostic {
                                    file: source_name.to_string(),
                                    pos,
                                    message,
                                }
                            }
                        };
                        eprintln!("{}", e);
                        had_error = true;
                    }
                }
            }
            _ => {}
        }
    }

    if had_error {
        process::exit(1);
    }

    if module_name.is_empty() {
        eprintln!("{}: no defmodule form found", input_file);
        process::exit(1);
    }

    let module_form = lower::lower_module_def(&module_name, &module_exports);
    let mut form_line_pairs: Vec<(sexp::types::SExp, usize)> = Vec::new();

    form_line_pairs.push((module_form, 1));

    for lf in &lowered_funs {
        form_line_pairs.push((lf.module_form.clone(), lf.line));
    }

    let eetf_bytes = eetf::encode_forms(&form_line_pairs);

    match output_file {
        Some(path) => {
            if let Err(e) = eetf::write_eetf_file(path, &eetf_bytes) {
                eprintln!("failed to write {}: {}", path, e);
                process::exit(1);
            }
        }
        None => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            if let Err(e) = handle.write_all(&eetf_bytes) {
                eprintln!("failed to write stdout: {}", e);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests;
