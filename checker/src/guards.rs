use crate::adt::{AdtDef, ReprKind};
use crate::error::Position;
use crate::lower::to_snake_case;
use crate::sexp::types::*;
use crate::type_env::TypeEnv;

pub fn guard_for_type(type_str: &str, var_name: &str, type_env: &TypeEnv) -> Option<SExp> {
    match type_str {
        "integer" => Some(guard_call("is_integer", var_name)),
        "float" => Some(guard_call("is_float", var_name)),
        "number" => Some(guard_call("is_number", var_name)),
        "binary" => Some(guard_call("is_binary", var_name)),
        "atom" => Some(guard_call("is_atom", var_name)),
        "boolean" => Some(guard_call("is_boolean", var_name)),
        "string" | "list" => Some(guard_call("is_list", var_name)),
        "map" => Some(guard_call("is_map", var_name)),
        "dynamic" => None,
        other => {
            if let Some(adt) = type_env.lookup_type(other) {
                guard_for_adt(adt, var_name, 28)
            } else {
                None
            }
        }
    }
}

fn guard_for_adt(adt: &AdtDef, var_name: &str, otp_version: u32) -> Option<SExp> {
    let repr = adt.effective_repr(otp_version);
    match repr {
        ReprKind::TaggedTuple => {
            if adt.is_all_nullary() {
                return guard_for_enum_adt(adt, var_name);
            }
            let has_nullary = adt.constructors.iter().any(|c| c.fields.is_empty());
            if has_nullary {
                // Mixed: some nullary (atoms) + some with fields (tuples)
                Some(SExp::List(List::new(
                    vec![
                        sym("orelse"),
                        guard_call("is_tuple", var_name),
                        guard_call("is_atom", var_name),
                    ],
                    dp(),
                )))
            } else {
                Some(guard_call("is_tuple", var_name))
            }
        }
        ReprKind::Enum => guard_for_enum_adt(adt, var_name),
        ReprKind::Transparent => {
            if let Some(ctor) = adt.constructors.first() {
                if let Some(field) = ctor.fields.first() {
                    return guard_for_type(&field.type_expr, var_name, &TypeEnv::new());
                }
            }
            None
        }
        ReprKind::NativeRecord | ReprKind::Default => Some(guard_call("is_tuple", var_name)),
    }
}

fn guard_for_enum_adt(adt: &AdtDef, var_name: &str) -> Option<SExp> {
    let tags: Vec<SExp> = adt
        .constructors
        .iter()
        .map(|c| quoted_atom(&to_snake_case(&c.name)))
        .collect();

    if tags.len() == 1 {
        return Some(SExp::List(List::new(
            vec![sym("=:="), sym(var_name), tags.into_iter().next().unwrap()],
            dp(),
        )));
    }

    let mut or_elems = vec![sym("orelse")];
    for tag in &tags {
        or_elems.push(SExp::List(List::new(
            vec![sym("=:="), sym(var_name), tag.clone()],
            dp(),
        )));
    }
    Some(SExp::List(List::new(or_elems, dp())))
}

pub fn type_error_term(
    fun_name: &str,
    arg_index: usize,
    _arg_name: &str,
    expected_type: &str,
    var_name: &str,
) -> SExp {
    SExp::List(List::new(
        vec![
            sym("tuple"),
            quoted_atom("type_error"),
            SExp::List(List::new(
                vec![
                    sym("list"),
                    SExp::List(List::new(
                        vec![
                            sym("tuple"),
                            quoted_atom("expected"),
                            quoted_atom(expected_type),
                        ],
                        dp(),
                    )),
                    SExp::List(List::new(
                        vec![sym("tuple"), quoted_atom("got"), sym(var_name)],
                        dp(),
                    )),
                    SExp::List(List::new(
                        vec![sym("tuple"), quoted_atom("function"), quoted_atom(fun_name)],
                        dp(),
                    )),
                    SExp::List(List::new(
                        vec![
                            sym("tuple"),
                            quoted_atom("arg"),
                            SExp::Number(Number::new((arg_index + 1).to_string(), dp())),
                        ],
                        dp(),
                    )),
                ],
                dp(),
            )),
        ],
        dp(),
    ))
}

fn guard_call(predicate: &str, var_name: &str) -> SExp {
    SExp::List(List::new(vec![sym(predicate), sym(var_name)], dp()))
}

fn sym(name: &str) -> SExp {
    SExp::Symbol(Symbol::new(name, dp()))
}

fn quoted_atom(name: &str) -> SExp {
    SExp::List(List::new(vec![sym("quote"), sym(name)], dp()))
}

fn dp() -> Position {
    Position::new(0, 0, 0)
}
