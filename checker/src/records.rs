use crate::adt::AdtDef;
use crate::error::Position;
use crate::guards;
use crate::lower::to_snake_case;
use crate::sexp::types::*;
use crate::type_env::TypeEnv;

pub fn generate_record_functions(adt: &AdtDef, type_env: &TypeEnv) -> Vec<(SExp, usize)> {
    let mut forms = Vec::new();
    let rec_name = &adt.name;
    let ctor = &adt.constructors[0];
    let tag = to_snake_case(rec_name);

    // make-<rec> constructor
    forms.push((generate_make(rec_name, ctor, &tag, type_env), adt.pos.line));

    // <rec>-<field> accessors
    for (i, field) in ctor.fields.iter().enumerate() {
        forms.push((
            generate_accessor(rec_name, &field.name, i, ctor.fields.len(), &tag),
            adt.pos.line,
        ));
    }

    // set-<rec>-<field> updaters
    for (i, field) in ctor.fields.iter().enumerate() {
        forms.push((
            generate_updater(
                rec_name,
                &field.name,
                &field.type_expr,
                i,
                ctor,
                &tag,
                type_env,
            ),
            adt.pos.line,
        ));
    }

    forms
}

pub fn record_exports(adt: &AdtDef) -> Vec<(String, usize)> {
    let mut exports = Vec::new();
    let rec_name = &adt.name;
    let ctor = &adt.constructors[0];

    exports.push((format!("make-{}", rec_name), ctor.fields.len()));

    for field in &ctor.fields {
        exports.push((format!("{}-{}", rec_name, field.name), 1));
        exports.push((format!("set-{}-{}", rec_name, field.name), 2));
    }

    exports
}

fn generate_make(
    rec_name: &str,
    ctor: &crate::adt::CtorDef,
    tag: &str,
    type_env: &TypeEnv,
) -> SExp {
    let fn_name = format!("make-{}", rec_name);
    let arg_names: Vec<String> = ctor.fields.iter().map(|f| f.name.clone()).collect();

    let guard_exprs: Vec<SExp> = ctor
        .fields
        .iter()
        .filter_map(|f| guards::guard_for_type(&f.type_expr, &f.name, type_env))
        .collect();

    let mut tuple_elems = vec![sym("tuple"), quoted_atom(tag)];
    for field in &ctor.fields {
        tuple_elems.push(sym(&field.name));
    }
    let result = SExp::List(List::new(tuple_elems, dp()));

    let func_body = if guard_exprs.is_empty() {
        let mut lambda = vec![
            sym("lambda"),
            SExp::List(List::new(arg_names.iter().map(|n| sym(n)).collect(), dp())),
        ];
        lambda.push(result);
        SExp::List(List::new(lambda, dp()))
    } else {
        let arg_syms: Vec<SExp> = arg_names.iter().map(|n| sym(n)).collect();
        let mut happy = vec![SExp::List(List::new(arg_syms.clone(), dp()))];

        let when_expr = if guard_exprs.len() == 1 {
            guard_exprs.into_iter().next().unwrap()
        } else {
            let mut and_elems = vec![sym("andalso")];
            and_elems.extend(guard_exprs);
            SExp::List(List::new(and_elems, dp()))
        };
        happy.push(SExp::List(List::new(vec![sym("when"), when_expr], dp())));
        happy.push(result);

        let fallback_args: Vec<SExp> = (0..ctor.fields.len())
            .map(|i| sym(&format!("-a{}-", i)))
            .collect();
        let error_term = guards::type_error_term(&fn_name, 0, "", rec_name, &format!("-a{}-", 0));
        let fallback = SExp::List(List::new(
            vec![
                SExp::List(List::new(fallback_args, dp())),
                SExp::List(List::new(vec![sym("error"), error_term], dp())),
            ],
            dp(),
        ));

        SExp::List(List::new(
            vec![
                sym("match-lambda"),
                SExp::List(List::new(happy, dp())),
                fallback,
            ],
            dp(),
        ))
    };

    SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(&fn_name),
            SExp::List(List::new(vec![], dp())),
            func_body,
        ],
        dp(),
    ))
}

fn generate_accessor(
    rec_name: &str,
    field_name: &str,
    field_index: usize,
    total_fields: usize,
    tag: &str,
) -> SExp {
    let fn_name = format!("{}-{}", rec_name, field_name);

    let mut pat_elems = vec![sym("tuple"), quoted_atom(tag)];
    for i in 0..total_fields {
        if i == field_index {
            pat_elems.push(sym("-val-"));
        } else {
            pat_elems.push(sym("_"));
        }
    }
    let pattern = SExp::List(List::new(pat_elems, dp()));

    let clause = SExp::List(List::new(
        vec![SExp::List(List::new(vec![pattern], dp())), sym("-val-")],
        dp(),
    ));

    let body = SExp::List(List::new(vec![sym("match-lambda"), clause], dp()));

    SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(&fn_name),
            SExp::List(List::new(vec![], dp())),
            body,
        ],
        dp(),
    ))
}

fn generate_updater(
    rec_name: &str,
    field_name: &str,
    field_type: &str,
    field_index: usize,
    ctor: &crate::adt::CtorDef,
    tag: &str,
    type_env: &TypeEnv,
) -> SExp {
    let fn_name = format!("set-{}-{}", rec_name, field_name);

    let mut pat_elems = vec![sym("tuple"), quoted_atom(tag)];
    for (i, _f) in ctor.fields.iter().enumerate() {
        pat_elems.push(sym(&format!("-f{}-", i)));
    }
    let rec_pattern = SExp::List(List::new(pat_elems, dp()));

    let mut new_tuple = vec![sym("tuple"), quoted_atom(tag)];
    for (i, _f) in ctor.fields.iter().enumerate() {
        if i == field_index {
            new_tuple.push(sym("-new-val-"));
        } else {
            new_tuple.push(sym(&format!("-f{}-", i)));
        }
    }
    let result = SExp::List(List::new(new_tuple, dp()));

    let guard = guards::guard_for_type(field_type, "-new-val-", type_env);

    let func_body = if let Some(guard_expr) = guard {
        let happy = SExp::List(List::new(
            vec![
                SExp::List(List::new(vec![rec_pattern.clone(), sym("-new-val-")], dp())),
                SExp::List(List::new(vec![sym("when"), guard_expr], dp())),
                result,
            ],
            dp(),
        ));

        let error_term = guards::type_error_term(&fn_name, 1, "new-val", field_type, "-new-val-");
        let fallback = SExp::List(List::new(
            vec![
                SExp::List(List::new(vec![sym("_rec"), sym("-new-val-")], dp())),
                SExp::List(List::new(vec![sym("error"), error_term], dp())),
            ],
            dp(),
        ));

        SExp::List(List::new(vec![sym("match-lambda"), happy, fallback], dp()))
    } else {
        let mut lambda = vec![
            sym("lambda"),
            SExp::List(List::new(vec![rec_pattern, sym("-new-val-")], dp())),
        ];
        lambda.push(result);
        SExp::List(List::new(lambda, dp()))
    };

    SExp::List(List::new(
        vec![
            sym("define-function"),
            sym(&fn_name),
            SExp::List(List::new(vec![], dp())),
            func_body,
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
