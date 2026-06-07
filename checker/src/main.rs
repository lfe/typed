mod adt;
mod eetf;
mod error;
mod lower;
mod sexp;
mod type_env;
mod typed_surface;

use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: typed-check <file.lfe> [--output <file.eetf>] [--otp-version <N>]");
        process::exit(2);
    }

    let input_file = &args[1];
    let output_file = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());
    let otp_version: u32 = args
        .iter()
        .position(|a| a == "--otp-version")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(28);

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
    let mut env = type_env::TypeEnv::new();
    let mut pending_forms: Vec<&sexp::types::SExp> = Vec::new();

    for form in &forms {
        if let Some(mdef) = typed_surface::extract_module_def(form) {
            module_name = mdef.name;
            module_exports = mdef.exports;
            continue;
        }

        if is_deftype(form) {
            match adt::extract_deftype(form) {
                Ok(adt_def) => {
                    env.register(adt_def);
                }
                Err(e) => {
                    let e = stamp_file(e, source_name);
                    eprintln!("{}", e);
                    had_error = true;
                }
            }
            continue;
        }

        pending_forms.push(form);
    }

    let ctor_names = env.all_ctor_names();

    for form in &pending_forms {
        if is_defun_typed(form) {
            match typed_surface::extract_typed_fun(form) {
                Ok(tf) => {
                    let body = lower_body_constructions(
                        &tf.body,
                        &env,
                        &ctor_names,
                        otp_version,
                        source_name,
                        &mut had_error,
                    );
                    let mut tf_lowered = tf.clone();
                    tf_lowered.body = body;
                    let lowered = lower::lower_typed_fun(&tf_lowered);
                    lowered_funs.push(lowered);
                }
                Err(e) => {
                    let e = stamp_file(e, source_name);
                    eprintln!("{}", e);
                    had_error = true;
                }
            }
            continue;
        }

        if let Some(result) = adt::extract_construction(form, &ctor_names) {
            match result {
                Ok(construction) => {
                    if let Some(adt_def) = env.lookup_ctor(&construction.ctor_name) {
                        if let Err(e) = adt::check_construction(&construction, adt_def, source_name)
                        {
                            eprintln!("{}", e);
                            had_error = true;
                        }
                    }
                }
                Err(e) => {
                    let e = stamp_file(e, source_name);
                    eprintln!("{}", e);
                    had_error = true;
                }
            }
        }
    }

    if had_error {
        process::exit(1);
    }

    if module_name.is_empty() {
        eprintln!("{}: no defmodule form found", input_file);
        process::exit(1);
    }

    let all_adts: Vec<_> = env.all_types().cloned().collect();
    let mut extra_attrs = Vec::new();
    if !all_adts.is_empty() {
        let registry = lower::lower_registry_attr(&all_adts);
        extra_attrs.push(("typed-registry".to_string(), registry));
    }

    let module_form =
        lower::lower_module_def_with_attrs(&module_name, &module_exports, &extra_attrs);
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

fn lower_body_constructions(
    body: &[sexp::types::SExp],
    env: &type_env::TypeEnv,
    ctor_names: &[String],
    otp_version: u32,
    source_name: &str,
    had_error: &mut bool,
) -> Vec<sexp::types::SExp> {
    body.iter()
        .map(|expr| {
            lower_expr_constructions(expr, env, ctor_names, otp_version, source_name, had_error)
        })
        .collect()
}

fn lower_expr_constructions(
    expr: &sexp::types::SExp,
    env: &type_env::TypeEnv,
    ctor_names: &[String],
    otp_version: u32,
    source_name: &str,
    had_error: &mut bool,
) -> sexp::types::SExp {
    // Check for known constructor calls
    if let Some(result) = adt::extract_construction(expr, ctor_names) {
        match result {
            Ok(construction) => {
                if let Some(adt_def) = env.lookup_ctor(&construction.ctor_name) {
                    if let Err(e) = adt::check_construction(&construction, adt_def, source_name) {
                        eprintln!("{}", e);
                        *had_error = true;
                        return expr.clone();
                    }
                    let ctor_def = adt_def.find_ctor(&construction.ctor_name).unwrap();
                    return lower::lower_construction(
                        &construction,
                        ctor_def,
                        adt_def,
                        otp_version,
                    );
                }
            }
            Err(e) => {
                let e = stamp_file(e, source_name);
                eprintln!("{}", e);
                *had_error = true;
                return expr.clone();
            }
        }
    }

    // Detect capitalized calls that look like constructor attempts but aren't known
    if let sexp::types::SExp::List(l) = expr {
        if let Some(sexp::types::SExp::Symbol(s)) = l.elements.first() {
            if s.value.starts_with(|c: char| c.is_uppercase()) && !ctor_names.contains(&s.value) {
                let e = error::CheckError::Diagnostic {
                    file: source_name.to_string(),
                    pos: s.pos,
                    message: format!(
                        "unknown constructor `{}`; no deftype declares this constructor",
                        s.value
                    ),
                };
                eprintln!("{}", e);
                *had_error = true;
                return expr.clone();
            }
        }

        let lowered_elems: Vec<_> = l
            .elements
            .iter()
            .map(|e| {
                lower_expr_constructions(e, env, ctor_names, otp_version, source_name, had_error)
            })
            .collect();
        return sexp::types::SExp::List(sexp::types::List::new(lowered_elems, l.pos));
    }

    expr.clone()
}

fn is_deftype(form: &sexp::types::SExp) -> bool {
    matches!(form, sexp::types::SExp::List(l)
        if !l.elements.is_empty()
            && matches!(&l.elements[0], sexp::types::SExp::Symbol(s) if s.value == "deftype"))
}

fn is_defun_typed(form: &sexp::types::SExp) -> bool {
    matches!(form, sexp::types::SExp::List(l)
        if !l.elements.is_empty()
            && matches!(&l.elements[0], sexp::types::SExp::Symbol(s) if s.value == "defun/typed"))
}

fn stamp_file(e: error::CheckError, file: &str) -> error::CheckError {
    match e {
        error::CheckError::Diagnostic { pos, message, .. } => error::CheckError::Diagnostic {
            file: file.to_string(),
            pos,
            message,
        },
    }
}

#[cfg(test)]
mod tests;
