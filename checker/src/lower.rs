use crate::adt::{AdtDef, Construction, CtorDef, ReprKind};
use crate::error::Position;
use crate::sexp::types::*;
use crate::typed_surface::TypedFun;

#[derive(Debug)]
pub struct LoweredForm {
    pub module_form: SExp,
    pub line: usize,
}

pub fn lower_typed_fun(tf: &TypedFun) -> LoweredForm {
    let arg_names: Vec<SExp> = tf.args.iter().map(|(name, _)| sym(name)).collect();

    let body_exprs: Vec<SExp> = tf.body.iter().map(strip_positions).collect();

    let mut lambda_elems = vec![sym("lambda"), SExp::List(List::new(arg_names, dp()))];
    lambda_elems.extend(body_exprs);

    let define_function = SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(&tf.name),
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(lambda_elems, dp())),
        ],
        dp(),
    ));

    LoweredForm {
        module_form: define_function,
        line: tf.pos.line,
    }
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
    if ctor_def.fields.is_empty() {
        return quoted_atom(&ctor_def.name);
    }
    let mut elems = vec![sym("tuple"), quoted_atom(&ctor_def.name)];
    for field_def in &ctor_def.fields {
        let val = construction
            .fields
            .iter()
            .find(|(n, _)| *n == field_def.name)
            .map(|(_, v)| strip_positions(v))
            .unwrap_or_else(|| SExp::Nil(Nil::new(dp())));
        elems.push(val);
    }
    SExp::List(List::new(elems, dp()))
}

fn lower_enum(construction: &Construction) -> SExp {
    quoted_atom(&construction.ctor_name.to_lowercase())
}

fn lower_transparent(construction: &Construction) -> SExp {
    construction
        .fields
        .first()
        .map(|(_, v)| strip_positions(v))
        .unwrap_or_else(|| SExp::Nil(Nil::new(dp())))
}

fn lower_native_record(construction: &Construction, ctor_def: &CtorDef) -> SExp {
    if ctor_def.fields.is_empty() {
        return quoted_atom(&ctor_def.name);
    }
    let mut elems = vec![sym("make-record"), quoted_atom(&ctor_def.name)];
    for field_def in &ctor_def.fields {
        let val = construction
            .fields
            .iter()
            .find(|(n, _)| *n == field_def.name)
            .map(|(_, v)| strip_positions(v))
            .unwrap_or_else(|| SExp::Nil(Nil::new(dp())));
        elems.push(quoted_atom(&field_def.name));
        elems.push(val);
    }
    SExp::List(List::new(elems, dp()))
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

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "will be used when emitting -type attrs in M1+")
)]
pub fn lower_erlang_type_attr(adt: &AdtDef) -> SExp {
    let mut union_parts = Vec::new();
    for ctor in &adt.constructors {
        if ctor.fields.is_empty() {
            union_parts.push(quoted_atom(&ctor.name));
        } else {
            let mut tuple_elems = vec![sym("tuple")];
            tuple_elems.push(quoted_atom(&ctor.name));
            for field in &ctor.fields {
                tuple_elems.push(sym(&field.type_expr));
            }
            union_parts.push(SExp::List(List::new(tuple_elems, dp())));
        }
    }
    if union_parts.len() == 1 {
        union_parts.into_iter().next().unwrap()
    } else {
        let mut union = vec![sym("union")];
        union.extend(union_parts);
        SExp::List(List::new(union, dp()))
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
