use crate::adt::{AdtDef, ReprKind};
use crate::error::Position;
use crate::lower::to_snake_case;
use crate::sexp::types::*;

pub fn generate_validator(adt: &AdtDef, otp_version: u32) -> SExp {
    let repr = adt.effective_repr(otp_version);
    let type_name = &adt.name;
    let fn_name = format!("validate-{}", type_name);

    match repr {
        ReprKind::Enum => generate_enum_validator(adt, &fn_name),
        ReprKind::Transparent => generate_transparent_validator(adt, &fn_name),
        ReprKind::TaggedTuple | ReprKind::Default => generate_tagged_tuple_validator(adt, &fn_name),
        ReprKind::NativeRecord => generate_tagged_tuple_validator(adt, &fn_name),
    }
}

pub fn generate_decode(adt: &AdtDef, _otp_version: u32) -> SExp {
    let type_name = &adt.name;
    let validate_fn = format!("validate-{}", type_name);
    let decode_fn = format!("decode-{}", type_name);

    let body = SExp::List(List::new(
        vec![
            sym(&validate_fn),
            sym("term"),
            SExp::List(List::new(vec![], dp())),
        ],
        dp(),
    ));

    make_define_function(&decode_fn, vec!["term".to_string()], body)
}

fn generate_enum_validator(adt: &AdtDef, fn_name: &str) -> SExp {
    let mut clauses = Vec::new();

    for ctor in &adt.constructors {
        let tag = to_snake_case(&ctor.name);
        let mut clause = vec![SExp::List(List::new(
            vec![quoted_atom(&tag), sym("_path")],
            dp(),
        ))];
        clause.push(SExp::List(List::new(
            vec![sym("tuple"), quoted_atom("ok"), quoted_atom(&tag)],
            dp(),
        )));
        clauses.push(SExp::List(List::new(clause, dp())));
    }

    let mut fallback = vec![SExp::List(List::new(vec![sym("other"), sym("path")], dp()))];
    fallback.push(make_error_result(&adt.name, "other", "path"));
    clauses.push(SExp::List(List::new(fallback, dp())));

    let mut ml = vec![sym("match-lambda")];
    ml.extend(clauses);

    make_define_function_raw(fn_name, SExp::List(List::new(ml, dp())))
}

fn generate_transparent_validator(adt: &AdtDef, fn_name: &str) -> SExp {
    let ctor = &adt.constructors[0];
    let field = &ctor.fields[0];
    let field_check = base_type_check(&field.type_expr, "term");

    let body = SExp::List(List::new(
        vec![
            sym("if"),
            field_check,
            SExp::List(List::new(
                vec![sym("tuple"), quoted_atom("ok"), sym("term")],
                dp(),
            )),
            make_error_result(&adt.name, "term", "path"),
        ],
        dp(),
    ));

    let mut ml_clauses = vec![sym("match-lambda")];
    let mut clause = vec![SExp::List(List::new(vec![sym("term"), sym("path")], dp()))];
    clause.push(body);
    ml_clauses.push(SExp::List(List::new(clause, dp())));

    make_define_function_raw(fn_name, SExp::List(List::new(ml_clauses, dp())))
}

fn generate_tagged_tuple_validator(adt: &AdtDef, fn_name: &str) -> SExp {
    let mut clauses = Vec::new();

    for ctor in &adt.constructors {
        let tag = to_snake_case(&ctor.name);

        if ctor.fields.is_empty() {
            let mut clause = vec![SExp::List(List::new(
                vec![quoted_atom(&tag), sym("_path")],
                dp(),
            ))];
            clause.push(SExp::List(List::new(
                vec![sym("tuple"), quoted_atom("ok"), quoted_atom(&tag)],
                dp(),
            )));
            clauses.push(SExp::List(List::new(clause, dp())));
        } else {
            let _arity = ctor.fields.len() + 1;
            let tuple_pat = make_tuple_pattern(&tag, &ctor.fields);

            let mut clause = vec![SExp::List(List::new(vec![tuple_pat, sym("path")], dp()))];

            let field_checks = make_field_checks(&tag, &ctor.fields);
            clause.push(field_checks);
            clauses.push(SExp::List(List::new(clause, dp())));
        }
    }

    let mut fallback = vec![SExp::List(List::new(vec![sym("other"), sym("path")], dp()))];
    fallback.push(make_error_result(&adt.name, "other", "path"));
    clauses.push(SExp::List(List::new(fallback, dp())));

    let mut ml = vec![sym("match-lambda")];
    ml.extend(clauses);

    make_define_function_raw(fn_name, SExp::List(List::new(ml, dp())))
}

fn make_tuple_pattern(tag: &str, fields: &[crate::adt::FieldDef]) -> SExp {
    let mut elems = vec![sym("tuple"), quoted_atom(tag)];
    for (i, _field) in fields.iter().enumerate() {
        elems.push(sym(&format!("-f{}-", i)));
    }
    SExp::List(List::new(elems, dp()))
}

fn make_field_checks(tag: &str, fields: &[crate::adt::FieldDef]) -> SExp {
    if fields.is_empty() {
        return SExp::List(List::new(
            vec![sym("tuple"), quoted_atom("ok"), sym("term")],
            dp(),
        ));
    }

    // Reconstruct the original tuple for the ok return value
    let mut ok_tuple_elems = vec![sym("tuple"), quoted_atom(tag)];
    for (i, _) in fields.iter().enumerate() {
        ok_tuple_elems.push(sym(&format!("-f{}-", i)));
    }
    let ok_value = SExp::List(List::new(ok_tuple_elems, dp()));

    let mut checks = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let var = format!("-f{}-", i);
        let check = base_type_check(&field.type_expr, &var);
        let field_path = SExp::List(List::new(
            vec![
                sym("++"),
                sym("path"),
                SExp::List(List::new(vec![sym("list"), quoted_atom(&field.name)], dp())),
            ],
            dp(),
        ));

        checks.push((
            var,
            check,
            field.name.clone(),
            field.type_expr.clone(),
            field_path,
        ));
    }

    let mut result = SExp::List(List::new(
        vec![sym("tuple"), quoted_atom("ok"), ok_value],
        dp(),
    ));

    for (var, check, _field_name, field_type, field_path) in checks.iter().rev() {
        result = SExp::List(List::new(
            vec![
                sym("if"),
                check.clone(),
                result,
                SExp::List(List::new(
                    vec![
                        sym("tuple"),
                        quoted_atom("error"),
                        SExp::List(List::new(
                            vec![
                                sym("tuple"),
                                quoted_atom("type_error"),
                                SExp::List(List::new(
                                    vec![
                                        sym("map"),
                                        quoted_atom("expected"),
                                        quoted_atom(field_type),
                                        quoted_atom("got"),
                                        sym(var),
                                        quoted_atom("path"),
                                        field_path.clone(),
                                    ],
                                    dp(),
                                )),
                            ],
                            dp(),
                        )),
                    ],
                    dp(),
                )),
            ],
            dp(),
        ));
    }

    // Wrap: we need to bind 'term' to reconstruct the original tuple for the ok case
    // Actually, the match-lambda already bound the fields, so we need the original value
    // Use a let to capture the whole term
    result
}

fn base_type_check(type_str: &str, var: &str) -> SExp {
    let pred = match type_str {
        "integer" => "is_integer",
        "float" => "is_float",
        "number" => "is_number",
        "binary" => "is_binary",
        "atom" => "is_atom",
        "boolean" => "is_boolean",
        "string" | "list" => "is_list",
        "map" => "is_map",
        _ => return quoted_atom("true"),
    };
    SExp::List(List::new(vec![sym(pred), sym(var)], dp()))
}

fn make_error_result(type_name: &str, var: &str, path_var: &str) -> SExp {
    SExp::List(List::new(
        vec![
            sym("tuple"),
            quoted_atom("error"),
            SExp::List(List::new(
                vec![
                    sym("tuple"),
                    quoted_atom("type_error"),
                    SExp::List(List::new(
                        vec![
                            sym("map"),
                            quoted_atom("expected"),
                            quoted_atom(type_name),
                            quoted_atom("got"),
                            sym(var),
                            quoted_atom("path"),
                            sym(path_var),
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

fn make_define_function(name: &str, args: Vec<String>, body: SExp) -> SExp {
    let arg_syms: Vec<SExp> = args.iter().map(|a| sym(a)).collect();
    let mut lambda = vec![sym("lambda"), SExp::List(List::new(arg_syms, dp()))];
    lambda.push(body);

    SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(name),
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(lambda, dp())),
        ],
        dp(),
    ))
}

#[expect(
    dead_code,
    reason = "deferred to M4.7: needs (call 'mod 'fun) EETF support"
)]
pub fn generate_render_helper() -> SExp {
    // (define-function render-type-error () (match-lambda
    //   ((#(type_error info))
    //    (let ((expected (maps:get 'expected info))
    //          (got (maps:get 'got info))
    //          (path (maps:get 'path info '())))
    //      (lists:flatten
    //        (io_lib:format "type error: expected ~p~s, got ~p"
    //          (list expected (render-path path) got)))))))
    let body = SExp::List(List::new(
        vec![
            sym("match-lambda"),
            SExp::List(List::new(
                vec![
                    SExp::List(List::new(
                        vec![SExp::List(List::new(
                            vec![sym("tuple"), quoted_atom("type_error"), sym("info")],
                            dp(),
                        ))],
                        dp(),
                    )),
                    SExp::List(List::new(
                        vec![
                            sym("let"),
                            SExp::List(List::new(
                                vec![
                                    SExp::List(List::new(
                                        vec![
                                            sym("expected"),
                                            SExp::List(List::new(
                                                vec![
                                                    sym("maps:get"),
                                                    quoted_atom("expected"),
                                                    sym("info"),
                                                ],
                                                dp(),
                                            )),
                                        ],
                                        dp(),
                                    )),
                                    SExp::List(List::new(
                                        vec![
                                            sym("got"),
                                            SExp::List(List::new(
                                                vec![
                                                    sym("maps:get"),
                                                    quoted_atom("got"),
                                                    sym("info"),
                                                ],
                                                dp(),
                                            )),
                                        ],
                                        dp(),
                                    )),
                                    SExp::List(List::new(
                                        vec![
                                            sym("path"),
                                            SExp::List(List::new(
                                                vec![
                                                    sym("maps:get"),
                                                    quoted_atom("path"),
                                                    sym("info"),
                                                    SExp::List(List::new(vec![], dp())),
                                                ],
                                                dp(),
                                            )),
                                        ],
                                        dp(),
                                    )),
                                ],
                                dp(),
                            )),
                            SExp::List(List::new(
                                vec![
                                    sym("lists:flatten"),
                                    SExp::List(List::new(
                                        vec![
                                            sym("io_lib:format"),
                                            SExp::String(StringLit::new(
                                                "type error: expected ~p~s, got ~p",
                                                dp(),
                                            )),
                                            SExp::List(List::new(
                                                vec![
                                                    sym("list"),
                                                    sym("expected"),
                                                    SExp::List(List::new(
                                                        vec![
                                                            sym("if"),
                                                            SExp::List(List::new(
                                                                vec![
                                                                    sym("=:="),
                                                                    sym("path"),
                                                                    SExp::List(List::new(
                                                                        vec![],
                                                                        dp(),
                                                                    )),
                                                                ],
                                                                dp(),
                                                            )),
                                                            SExp::String(StringLit::new("", dp())),
                                                            SExp::List(List::new(
                                                                vec![
                                                                    sym("lists:flatten"),
                                                                    SExp::List(List::new(
                                                                        vec![
                                                                            sym("io_lib:format"),
                                                                            SExp::String(
                                                                                StringLit::new(
                                                                                    " at ~p",
                                                                                    dp(),
                                                                                ),
                                                                            ),
                                                                            SExp::List(List::new(
                                                                                vec![
                                                                                    sym("list"),
                                                                                    sym("path"),
                                                                                ],
                                                                                dp(),
                                                                            )),
                                                                        ],
                                                                        dp(),
                                                                    )),
                                                                ],
                                                                dp(),
                                                            )),
                                                        ],
                                                        dp(),
                                                    )),
                                                    sym("got"),
                                                ],
                                                dp(),
                                            )),
                                        ],
                                        dp(),
                                    )),
                                ],
                                dp(),
                            )),
                        ],
                        dp(),
                    )),
                ],
                dp(),
            )),
        ],
        dp(),
    ));

    make_define_function_raw("render-type-error", body)
}

fn make_define_function_raw(name: &str, body: SExp) -> SExp {
    SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(name),
            SExp::List(List::new(vec![], dp())),
            body,
        ],
        dp(),
    ))
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
