use crate::adt::{AdtDef, Construction, CtorDef, ReprKind};
use crate::error::Position;
use crate::guards;
use crate::sexp::types::*;
use crate::type_env::TypeEnv;
use crate::typed_surface::TypedFun;

#[derive(Debug)]
pub struct LoweredForm {
    pub module_form: SExp,
    pub line: usize,
}

pub fn lower_typed_fun(tf: &TypedFun, type_env: &TypeEnv) -> LoweredForm {
    let arg_names: Vec<SExp> = tf.args.iter().map(|(name, _)| sym(name)).collect();
    let body_exprs: Vec<SExp> = tf.body.iter().map(strip_positions).collect();

    let guard_exprs: Vec<SExp> = tf
        .args
        .iter()
        .filter_map(|(name, type_str)| guards::guard_for_type(type_str, name, type_env))
        .collect();

    let func_body = if guard_exprs.is_empty() {
        let mut lambda_elems = vec![sym("lambda"), SExp::List(List::new(arg_names, dp()))];
        lambda_elems.extend(body_exprs);
        SExp::List(List::new(lambda_elems, dp()))
    } else {
        let mut happy_clause = vec![SExp::List(List::new(arg_names.clone(), dp()))];
        let when_expr = if guard_exprs.len() == 1 {
            guard_exprs.into_iter().next().unwrap()
        } else {
            let mut and_elems = vec![sym("andalso")];
            and_elems.extend(guard_exprs);
            SExp::List(List::new(and_elems, dp()))
        };
        happy_clause.push(SExp::List(List::new(vec![sym("when"), when_expr], dp())));
        happy_clause.extend(body_exprs);
        let happy = SExp::List(List::new(happy_clause, dp()));

        let fallback_args: Vec<SExp> = tf
            .args
            .iter()
            .enumerate()
            .map(|(i, _)| sym(&format!("-arg{}-", i)))
            .collect();

        let first_bad = build_first_bad_arg_check(tf, type_env);

        let mut fallback_clause = vec![SExp::List(List::new(fallback_args, dp()))];
        fallback_clause.push(first_bad);
        let fallback = SExp::List(List::new(fallback_clause, dp()));

        SExp::List(List::new(vec![sym("match-lambda"), happy, fallback], dp()))
    };

    let define_function = SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(&tf.name),
            SExp::List(List::new(vec![], dp())),
            func_body,
        ],
        dp(),
    ));

    LoweredForm {
        module_form: define_function,
        line: tf.pos.line,
    }
}

fn build_first_bad_arg_check(tf: &TypedFun, type_env: &TypeEnv) -> SExp {
    for (i, (name, type_str)) in tf.args.iter().enumerate() {
        if guards::guard_for_type(type_str, &format!("-arg{}-", i), type_env).is_some() {
            let error_term =
                guards::type_error_term(&tf.name, i, name, type_str, &format!("-arg{}-", i));
            return SExp::List(List::new(vec![sym("error"), error_term], dp()));
        }
    }
    SExp::List(List::new(
        vec![sym("error"), quoted_atom("type_error")],
        dp(),
    ))
}

pub fn lower_construction(
    construction: &Construction,
    ctor_def: &CtorDef,
    adt: &AdtDef,
    otp_version: u32,
) -> SExp {
    let repr = adt.effective_repr(otp_version);
    match repr {
        ReprKind::TaggedTuple => lower_tagged_tuple(construction, ctor_def),
        ReprKind::Enum => lower_enum(construction),
        ReprKind::Transparent => lower_transparent(construction),
        ReprKind::NativeRecord => lower_native_record(construction, ctor_def),
        ReprKind::Default => lower_tagged_tuple(construction, ctor_def),
    }
}

fn lower_tagged_tuple(construction: &Construction, ctor_def: &CtorDef) -> SExp {
    let tag = to_snake_case(&ctor_def.name);
    if ctor_def.fields.is_empty() {
        return quoted_atom(&tag);
    }
    let mut elems = vec![sym("tuple"), quoted_atom(&tag)];
    for field_def in &ctor_def.fields {
        let val = find_field_value(construction, &field_def.name);
        elems.push(val);
    }
    SExp::List(List::new(elems, dp()))
}

fn lower_enum(construction: &Construction) -> SExp {
    quoted_atom(&to_snake_case(&construction.ctor_name))
}

fn lower_transparent(construction: &Construction) -> SExp {
    construction
        .fields
        .first()
        .map(|(_, v)| strip_positions(v))
        .unwrap_or_else(|| SExp::Nil(Nil::new(dp())))
}

fn lower_native_record(construction: &Construction, ctor_def: &CtorDef) -> SExp {
    let tag = to_snake_case(&ctor_def.name);
    if ctor_def.fields.is_empty() {
        return quoted_atom(&tag);
    }
    let mut elems = vec![sym("make-record"), quoted_atom(&tag)];
    for field_def in &ctor_def.fields {
        let val = find_field_value(construction, &field_def.name);
        elems.push(quoted_atom(&field_def.name));
        elems.push(val);
    }
    SExp::List(List::new(elems, dp()))
}

fn find_field_value(construction: &Construction, field_name: &str) -> SExp {
    construction
        .fields
        .iter()
        .find(|(n, _)| *n == field_name)
        .map(|(_, v)| strip_positions(v))
        .unwrap_or_else(|| SExp::Nil(Nil::new(dp())))
}

pub fn to_snake_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    let mut prev_was_upper = false;
    let mut prev_was_separator = true;
    for (i, ch) in name.chars().enumerate() {
        if ch == '-' || ch == '_' {
            result.push('_');
            prev_was_upper = false;
            prev_was_separator = true;
            continue;
        }
        if ch.is_uppercase() {
            let next_is_lower = name[i + ch.len_utf8()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_lowercase());
            if !prev_was_separator && (!prev_was_upper || next_is_lower) {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_was_upper = true;
        } else {
            result.push(ch);
            prev_was_upper = false;
        }
        prev_was_separator = false;
    }
    result
}

#[cfg(test)]
pub fn lower_module_def(name: &str, exports: &[(String, usize)]) -> SExp {
    lower_module_def_with_attrs(name, exports, &[])
}

pub fn lower_module_def_with_attrs(
    name: &str,
    exports: &[(String, usize)],
    extra_attrs: &[(String, SExp)],
) -> SExp {
    let mut export_pairs = Vec::new();
    for (fname, arity) in exports {
        export_pairs.push(SExp::List(List::new(
            vec![sym(fname), num(&arity.to_string())],
            dp(),
        )));
    }

    let mut export_elems = vec![sym("export")];
    export_elems.extend(export_pairs);
    let export_attr = SExp::List(List::new(export_elems, dp()));

    let mut attr_list = vec![export_attr];
    for (attr_name, attr_value) in extra_attrs {
        attr_list.push(SExp::List(List::new(
            vec![sym(attr_name), attr_value.clone()],
            dp(),
        )));
    }

    let attrs = SExp::List(List::new(attr_list, dp()));
    let metas = SExp::List(List::new(vec![], dp()));

    SExp::List(List::new(
        vec![sym("define-module"), sym(name), metas, attrs],
        dp(),
    ))
}

pub fn lower_registry_attr(adts: &[AdtDef]) -> SExp {
    let mut type_entries = Vec::new();
    for adt in adts {
        let mut ctor_entries = Vec::new();
        for ctor in &adt.constructors {
            let mut field_entries = Vec::new();
            for field in &ctor.fields {
                field_entries.push(SExp::List(List::new(
                    vec![sym(&field.name), sym(&field.type_expr)],
                    dp(),
                )));
            }
            ctor_entries.push(SExp::List(List::new(
                vec![sym(&ctor.name), SExp::List(List::new(field_entries, dp()))],
                dp(),
            )));
        }

        let repr_sym = match &adt.repr {
            ReprKind::TaggedTuple => "tagged-tuple",
            ReprKind::Enum => "enum",
            ReprKind::Transparent => "transparent",
            ReprKind::NativeRecord => "native-record",
            ReprKind::Default => "default",
        };

        let mut params = Vec::new();
        for p in &adt.type_params {
            params.push(sym(p));
        }

        type_entries.push(SExp::List(List::new(
            vec![
                sym(&adt.name),
                SExp::List(List::new(params, dp())),
                sym(repr_sym),
                SExp::List(List::new(ctor_entries, dp())),
            ],
            dp(),
        )));
    }
    SExp::List(List::new(type_entries, dp()))
}

#[cfg(test)]
pub fn deserialize_registry_entry(entry: &SExp) -> Result<AdtDef, String> {
    let list = match entry {
        SExp::List(l) => l,
        _ => return Err("registry entry must be a list".to_string()),
    };
    if list.elements.len() != 4 {
        return Err(format!(
            "registry entry needs 4 elements (name params repr ctors), got {}",
            list.elements.len()
        ));
    }

    let name = match &list.elements[0] {
        SExp::Symbol(s) => s.value.clone(),
        _ => return Err("registry entry[0] must be a type name symbol".to_string()),
    };

    let type_params = match &list.elements[1] {
        SExp::List(l) => l
            .elements
            .iter()
            .map(|e| match e {
                SExp::Symbol(s) => Ok(s.value.clone()),
                _ => Err("type param must be a symbol".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("registry entry[1] must be a params list".to_string()),
    };

    let repr = match &list.elements[2] {
        SExp::Symbol(s) => match s.value.as_str() {
            "tagged-tuple" => ReprKind::TaggedTuple,
            "enum" => ReprKind::Enum,
            "transparent" => ReprKind::Transparent,
            "native-record" => ReprKind::NativeRecord,
            "default" => ReprKind::Default,
            other => return Err(format!("unknown repr: {other}")),
        },
        _ => return Err("registry entry[2] must be a repr symbol".to_string()),
    };

    let constructors = match &list.elements[3] {
        SExp::List(ctors_list) => ctors_list
            .elements
            .iter()
            .map(deserialize_ctor)
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("registry entry[3] must be a constructors list".to_string()),
    };

    Ok(AdtDef {
        name,
        type_params,
        constructors,
        repr,
        pos: Position::new(0, 0, 0),
    })
}

#[cfg(test)]
fn deserialize_ctor(sexp: &SExp) -> Result<CtorDef, String> {
    let list = match sexp {
        SExp::List(l) => l,
        _ => return Err("constructor entry must be a list".to_string()),
    };
    if list.elements.len() != 2 {
        return Err(format!(
            "constructor needs 2 elements (name fields), got {}",
            list.elements.len()
        ));
    }

    let name = match &list.elements[0] {
        SExp::Symbol(s) => s.value.clone(),
        _ => return Err("constructor name must be a symbol".to_string()),
    };

    let fields = match &list.elements[1] {
        SExp::List(fields_list) => fields_list
            .elements
            .iter()
            .map(|f| {
                let fl = match f {
                    SExp::List(l) => l,
                    _ => return Err("field must be a (name type) list".to_string()),
                };
                if fl.elements.len() != 2 {
                    return Err(format!("field needs 2 elements, got {}", fl.elements.len()));
                }
                let field_name = match &fl.elements[0] {
                    SExp::Symbol(s) => s.value.clone(),
                    _ => return Err("field name must be a symbol".to_string()),
                };
                let field_type = match &fl.elements[1] {
                    SExp::Symbol(s) => s.value.clone(),
                    _ => return Err("field type must be a symbol".to_string()),
                };
                Ok(crate::adt::FieldDef {
                    name: field_name,
                    type_expr: field_type,
                    pos: Position::new(0, 0, 0),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("fields must be a list".to_string()),
    };

    Ok(CtorDef {
        name,
        fields,
        pos: Position::new(0, 0, 0),
    })
}

pub fn expand_quasiquotes(sexp: &SExp) -> SExp {
    match sexp {
        SExp::List(l) if l.elements.len() == 2 => {
            if let SExp::Symbol(s) = &l.elements[0] {
                if s.value == "backquote" {
                    return qq_expand(&l.elements[1]);
                }
            }
            let elems = l.elements.iter().map(expand_quasiquotes).collect();
            SExp::List(List::new(elems, l.pos))
        }
        SExp::List(l) => {
            let elems = l.elements.iter().map(expand_quasiquotes).collect();
            SExp::List(List::new(elems, l.pos))
        }
        other => other.clone(),
    }
}

fn qq_expand(template: &SExp) -> SExp {
    match template {
        SExp::List(l) if l.elements.len() == 2 => {
            if let SExp::Symbol(s) = &l.elements[0] {
                if s.value == "comma" {
                    return expand_quasiquotes(&l.elements[1]);
                }
            }
            qq_expand_list(&l.elements)
        }
        SExp::List(l) => qq_expand_list(&l.elements),
        SExp::Symbol(s) => SExp::List(List::new(vec![sym("quote"), SExp::Symbol(s.clone())], dp())),
        SExp::Tuple(t) => {
            let expanded: Vec<SExp> = t.elements.iter().map(qq_expand).collect();
            let mut elems = vec![sym("tuple")];
            elems.extend(expanded);
            SExp::List(List::new(elems, dp()))
        }
        other => other.clone(),
    }
}

fn qq_expand_list(elements: &[SExp]) -> SExp {
    if elements.is_empty() {
        return SExp::Nil(Nil::new(dp()));
    }

    let mut parts: Vec<SExp> = Vec::new();
    let mut has_splice = false;

    for elem in elements {
        if let SExp::List(l) = elem {
            if l.elements.len() == 2 {
                if let SExp::Symbol(s) = &l.elements[0] {
                    if s.value == "comma-at" {
                        has_splice = true;
                        break;
                    }
                }
            }
        }
    }

    if has_splice {
        for elem in elements {
            if let SExp::List(l) = elem {
                if l.elements.len() == 2 {
                    if let SExp::Symbol(s) = &l.elements[0] {
                        if s.value == "comma-at" {
                            parts.push(expand_quasiquotes(&l.elements[1]));
                            continue;
                        }
                    }
                }
            }
            parts.push(SExp::List(List::new(
                vec![sym("list"), qq_expand(elem)],
                dp(),
            )));
        }
        let mut result = parts.pop().unwrap_or(SExp::Nil(Nil::new(dp())));
        while let Some(part) = parts.pop() {
            result = SExp::List(List::new(vec![sym("++"), part, result], dp()));
        }
        result
    } else {
        let mut elems = vec![sym("list")];
        for elem in elements {
            elems.push(qq_expand(elem));
        }
        SExp::List(List::new(elems, dp()))
    }
}

fn strip_positions(sexp: &SExp) -> SExp {
    match sexp {
        SExp::Symbol(s) => SExp::Symbol(Symbol::new(s.value.clone(), dp())),
        SExp::Keyword(k) => SExp::Keyword(Keyword::new(k.name.clone(), dp())),
        SExp::String(s) => SExp::String(StringLit::new(s.value.clone(), dp())),
        SExp::Number(n) => SExp::Number(Number::new(n.value.clone(), dp())),
        SExp::Nil(_) => SExp::Nil(Nil::new(dp())),
        SExp::List(l) => {
            let elems = l.elements.iter().map(strip_positions).collect();
            SExp::List(List::new(elems, dp()))
        }
        SExp::Tuple(t) => {
            let elems = t.elements.iter().map(strip_positions).collect();
            SExp::Tuple(Tuple::new(elems, dp()))
        }
    }
}

fn sym(name: &str) -> SExp {
    SExp::Symbol(Symbol::new(name, dp()))
}

fn num(val: &str) -> SExp {
    SExp::Number(Number::new(val, dp()))
}

fn quoted_atom(name: &str) -> SExp {
    SExp::List(List::new(vec![sym("quote"), sym(name)], dp()))
}

fn dp() -> Position {
    Position::new(0, 0, 0)
}
