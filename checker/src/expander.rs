use std::cell::Cell;

use crate::error::Position;
use crate::sexp::types::*;

thread_local! {
    static VC: Cell<usize> = const { Cell::new(0) };
    static FC: Cell<usize> = const { Cell::new(0) };
    static MODULE_NAME: Cell<Option<String>> = const { Cell::new(None) };
}

fn new_var_name() -> String {
    VC.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("|-{}-|", n)
    })
}

fn new_fun_name() -> String {
    FC.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("do$^{}", n)
    })
}

pub fn reset_counters() {
    VC.with(|c| c.set(0));
    FC.with(|c| c.set(0));
}

pub fn set_module_name(name: &str) {
    MODULE_NAME.with(|c| c.set(Some(name.to_string())));
}

fn dp() -> Position {
    Position::new(0, 0, 0)
}

fn sym(name: &str) -> SExp {
    SExp::Symbol(Symbol::new(name, dp()))
}

fn quoted(s: &SExp) -> SExp {
    SExp::List(List::new(vec![sym("quote"), s.clone()], dp()))
}

fn is_backquote(s: &SExp) -> bool {
    matches!(s, SExp::Symbol(s) if s.value == "backquote")
}

fn is_comma(s: &SExp) -> bool {
    matches!(s, SExp::Symbol(s) if s.value == "comma")
}

fn is_comma_at(s: &SExp) -> bool {
    matches!(s, SExp::Symbol(s) if s.value == "comma-at")
}

pub fn expand_form(form: &SExp) -> SExp {
    match form {
        SExp::List(l) if l.elements.len() == 2 && is_backquote(&l.elements[0]) => {
            exp_backquote(&l.elements[1], 0)
        }
        SExp::List(l) => {
            let expanded: Vec<SExp> = l.elements.iter().map(expand_form).collect();
            let result = SExp::List(List::new(expanded, l.pos));
            expand_core_form(&result)
        }
        SExp::Tuple(t) => {
            let expanded: Vec<SExp> = t.elements.iter().map(expand_form).collect();
            SExp::Tuple(Tuple::new(expanded, t.pos))
        }
        other => other.clone(),
    }
}

fn expand_core_form(form: &SExp) -> SExp {
    let l = match form {
        SExp::List(l) if !l.elements.is_empty() => l,
        _ => return form.clone(),
    };
    let head = match &l.elements[0] {
        SExp::Symbol(s) => s.value.as_str(),
        _ => return form.clone(),
    };
    match head {
        "backquote" if l.elements.len() == 2 => exp_backquote(&l.elements[1], 0),
        "quote"
        | "define-module"
        | "define-record"
        | "define-function"
        | "define-macro"
        | "define-type"
        | "define-function-spec" => form.clone(),
        "case" if l.elements.len() >= 3 => expand_case(l),
        "let" if l.elements.len() >= 3 => expand_let(l),
        "let*" if l.elements.len() >= 3 => expand_let_star(l),
        "lambda" if l.elements.len() >= 3 => expand_lambda(l),
        "match-lambda" if l.elements.len() >= 2 => expand_match_lambda(l),
        "progn" => expand_progn(l),
        "if" => expand_if(l),
        "try" => expand_try(l),
        "receive" => expand_receive(l),
        "lc" | "bc" => expand_comprehension(l),
        "let-function" | "letrec-function" => expand_let_function(l),
        "defun" if l.elements.len() >= 4 => expand_defun(l),
        "defmodule" if l.elements.len() >= 3 => expand_defmodule(l),
        "defmacro" if l.elements.len() >= 4 => expand_defmacro(l),
        "cond" if l.elements.len() >= 2 => expand_cond(l),
        // P-1: c*r macros
        "caar" | "cadr" | "cdar" | "cddr" | "caaar" | "caadr" | "cadar" | "caddr" | "cdaar"
        | "cdadr" | "cddar" | "cdddr"
            if l.elements.len() == 2 =>
        {
            expand_cxr(head, &l.elements[1])
        }
        // P-2: list* → cons chain
        "list*" if l.elements.len() >= 2 => expand_list_star(&l.elements[1..]),
        // P-2: let* → nested let
        // (already handled above via "let*")
        // P-2: flet* → nested let-function
        "flet*" if l.elements.len() >= 3 => expand_flet_star(l),
        // P-2: do → letrec-function with gensym loop name
        "do" if l.elements.len() >= 3 => expand_do(l),
        // P-2: fun → lambda with gensym args (fun mod:fun arity or fun mod :fun arity)
        "fun" if l.elements.len() == 3 || l.elements.len() == 4 => expand_fun_ref(l),
        // P-2: ? → receive with timeout
        "?" if l.elements.len() >= 2 => expand_receive_timeout(l),
        // P-3: MODULE
        "MODULE" if l.elements.len() == 1 => {
            let name = MODULE_NAME.with(|c| {
                let val = c.take();
                let result = val.clone().unwrap_or_else(|| "undefined".to_string());
                c.set(val);
                result
            });
            quoted(&sym(&name))
        }
        // P-3: LINE
        "LINE" if l.elements.len() == 1 => {
            quoted(&SExp::Number(Number::new(l.pos.line.to_string(), dp())))
        }
        // P-3: colon-call (mod:fun ...) detected at parse level as (mod :fun ...)
        // Handled in expand_form for the Symbol+Keyword pattern
        // P-4: flet → let-function
        "flet" if l.elements.len() >= 3 => expand_flet(l),
        // P-4: fletrec → letrec-function
        "fletrec" if l.elements.len() >= 3 => expand_fletrec(l),
        // P-4: macrolet — expand bodies with local macros (Tier-1: just expand the body)
        "macrolet" if l.elements.len() >= 3 => expand_macrolet(l),
        // P-4: deftype → define-type
        "deftype" if l.elements.len() >= 2 => expand_deftype(l),
        // P-4: defspec → define-function-spec
        "defspec" if l.elements.len() >= 3 => expand_defspec(l),
        _ => {
            if l.elements.len() >= 3 {
                if let (SExp::Symbol(mod_s), SExp::Keyword(fun_k)) =
                    (&l.elements[0], &l.elements[1])
                {
                    if !matches!(
                        fun_k.name.as_str(),
                        "args" | "returns" | "body" | "type" | "module" | "export"
                    ) && mod_s.value.chars().next().is_some_and(|c| c.is_lowercase())
                    {
                        let mut call_elems = vec![
                            sym("call"),
                            quoted(&SExp::Symbol(mod_s.clone())),
                            quoted(&SExp::Symbol(Symbol::new(&fun_k.name, fun_k.pos))),
                        ];
                        for arg in &l.elements[2..] {
                            call_elems.push(expand_form(arg));
                        }
                        return SExp::List(List::new(call_elems, l.pos));
                    }
                }
            }
            form.clone()
        }
    }
}

// ============================================================
// Backquote expansion — faithful port of lfe_macro:exp_backquote
// ============================================================

fn exp_backquote(exp: &SExp, n: usize) -> SExp {
    match exp {
        SExp::List(l) if l.elements.len() == 2 && is_backquote(&l.elements[0]) => {
            SExp::List(List::new(
                vec![
                    sym("list"),
                    quoted(&sym("backquote")),
                    exp_backquote(&l.elements[1], n + 1),
                ],
                dp(),
            ))
        }
        SExp::List(l) if l.elements.len() == 2 && is_comma(&l.elements[0]) && n == 0 => {
            l.elements[1].clone()
        }
        SExp::List(l) if l.elements.len() == 2 && is_comma(&l.elements[0]) && n > 0 => exp_bq_cons(
            quoted(&sym("comma")),
            exp_backquote(
                &SExp::List(List::new(l.elements[1..].to_vec(), dp())),
                n - 1,
            ),
        ),
        SExp::List(l) if l.elements.len() == 2 && is_comma_at(&l.elements[0]) && n > 0 => {
            exp_bq_cons(
                quoted(&sym("comma-at")),
                exp_backquote(
                    &SExp::List(List::new(l.elements[1..].to_vec(), dp())),
                    n - 1,
                ),
            )
        }
        SExp::List(l)
            if !l.elements.is_empty()
                && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "map") =>
        {
            let kvs = &l.elements[1..];
            let expanded_kvs: Vec<SExp> = kvs.iter().map(|e| exp_backquote(e, n)).collect();
            let mut map_elems = vec![sym("map")];
            map_elems.extend(expanded_kvs);
            SExp::List(List::new(map_elems, dp()))
        }
        SExp::List(l) if has_dot_tail(l) => {
            let dot_pos = l
                .elements
                .iter()
                .position(|e| matches!(e, SExp::Symbol(s) if s.value == "."))
                .unwrap();
            let head_part = &l.elements[..dot_pos];
            let tail_part = &l.elements[dot_pos + 1..];
            if head_part.len() == 1 && tail_part.len() == 1 {
                let expanded_head = exp_backquote(&head_part[0], n);
                let expanded_tail = exp_backquote(&tail_part[0], n);
                exp_bq_cons(expanded_head, expanded_tail)
            } else {
                let mut result = if tail_part.len() == 1 {
                    exp_backquote(&tail_part[0], n)
                } else {
                    exp_backquote(&SExp::List(List::new(tail_part.to_vec(), dp())), n)
                };
                for elem in head_part.iter().rev() {
                    result = exp_bq_cons(exp_backquote(elem, n), result);
                }
                result
            }
        }
        SExp::List(l) if l.elements.len() >= 2 => {
            let head = &l.elements[0];
            let tail_elems = &l.elements[1..];

            if n == 0 {
                if let SExp::List(head_l) = head {
                    if head_l.elements.len() == 2 && is_comma(&head_l.elements[0]) {
                        let tail =
                            exp_backquote(&SExp::List(List::new(tail_elems.to_vec(), dp())), 0);
                        let mut append_args = vec![sym("list")];
                        append_args.push(head_l.elements[1].clone());
                        return exp_bq_append(SExp::List(List::new(append_args, dp())), tail);
                    }
                    if head_l.elements.len() == 2 && is_comma_at(&head_l.elements[0]) {
                        let tail =
                            exp_backquote(&SExp::List(List::new(tail_elems.to_vec(), dp())), 0);
                        let mut append_args = vec![sym("++")];
                        append_args.push(head_l.elements[1].clone());
                        return exp_bq_append(SExp::List(List::new(append_args, dp())), tail);
                    }
                }
            }

            let expanded_head = exp_backquote(head, n);
            let expanded_tail = exp_backquote(&SExp::List(List::new(tail_elems.to_vec(), dp())), n);
            exp_bq_cons(expanded_head, expanded_tail)
        }
        SExp::List(l) if l.elements.len() == 1 => {
            let expanded_head = exp_backquote(&l.elements[0], n);
            exp_bq_cons(expanded_head, SExp::Nil(Nil::new(dp())))
        }
        SExp::List(l) if l.elements.is_empty() => SExp::Nil(Nil::new(dp())),
        SExp::List(_) => exp.clone(),
        SExp::Tuple(t) => {
            let as_list = exp_backquote(&SExp::List(List::new(t.elements.clone(), dp())), n);
            match &as_list {
                SExp::List(l)
                    if !l.elements.is_empty()
                        && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "list") =>
                {
                    let mut elems = vec![sym("tuple")];
                    elems.extend(l.elements[1..].to_vec());
                    SExp::List(List::new(elems, dp()))
                }
                SExp::List(l)
                    if !l.elements.is_empty()
                        && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "cons") =>
                {
                    SExp::List(List::new(vec![sym("list_to_tuple"), as_list.clone()], dp()))
                }
                SExp::Nil(_) => SExp::List(List::new(vec![sym("tuple")], dp())),
                _ => SExp::List(List::new(vec![sym("list_to_tuple"), as_list.clone()], dp())),
            }
        }
        SExp::Symbol(s) => quoted(&SExp::Symbol(s.clone())),
        SExp::Keyword(k) => quoted(&SExp::Keyword(k.clone())),
        SExp::Nil(_) => exp.clone(),
        SExp::Number(_) | SExp::String(_) => exp.clone(),
    }
}

fn has_dot_tail(l: &List) -> bool {
    l.elements
        .iter()
        .any(|e| matches!(e, SExp::Symbol(s) if s.value == "."))
}

fn exp_bq_cons(l: SExp, r: SExp) -> SExp {
    match (&l, &r) {
        (SExp::List(ll), SExp::List(rl))
            if ll.elements.len() == 2
                && matches!(&ll.elements[0], SExp::Symbol(s) if s.value == "quote")
                && rl.elements.len() == 2
                && matches!(&rl.elements[0], SExp::Symbol(s) if s.value == "quote") =>
        {
            let mut pair = vec![ll.elements[1].clone()];
            if let SExp::List(inner) = &rl.elements[1] {
                pair.extend(inner.elements.clone());
            } else {
                pair.push(rl.elements[1].clone());
            }
            SExp::List(List::new(
                vec![sym("quote"), SExp::List(List::new(pair, dp()))],
                dp(),
            ))
        }
        (_, SExp::List(rl))
            if !rl.elements.is_empty()
                && matches!(&rl.elements[0], SExp::Symbol(s) if s.value == "list") =>
        {
            let mut elems = vec![sym("list"), l];
            elems.extend(rl.elements[1..].to_vec());
            SExp::List(List::new(elems, dp()))
        }
        (_, SExp::Nil(_)) => SExp::List(List::new(vec![sym("list"), l], dp())),
        _ => SExp::List(List::new(vec![sym("cons"), l, r], dp())),
    }
}

fn exp_bq_append(l: SExp, r: SExp) -> SExp {
    if let SExp::List(ll) = &l {
        if ll.elements.len() == 2 && matches!(&ll.elements[0], SExp::Symbol(s) if s.value == "++") {
            return exp_bq_append(ll.elements[1].clone(), r);
        }
    }
    match (&l, &r) {
        (SExp::Nil(_), _) => r,
        (_, SExp::Nil(_)) => l,
        (SExp::List(ll), SExp::List(rl))
            if ll.elements.len() == 2
                && matches!(&ll.elements[0], SExp::Symbol(s) if s.value == "list")
                && !rl.elements.is_empty()
                && matches!(&rl.elements[0], SExp::Symbol(s) if s.value == "list") =>
        {
            let mut elems = vec![sym("list"), ll.elements[1].clone()];
            elems.extend(rl.elements[1..].to_vec());
            SExp::List(List::new(elems, dp()))
        }
        (SExp::List(ll), _)
            if ll.elements.len() == 2
                && matches!(&ll.elements[0], SExp::Symbol(s) if s.value == "list") =>
        {
            SExp::List(List::new(
                vec![sym("cons"), ll.elements[1].clone(), r],
                dp(),
            ))
        }
        _ => SExp::List(List::new(vec![sym("++"), l, r], dp())),
    }
}

// ============================================================
// Core-form recursion — expand nested macros in all positions
// ============================================================

fn expand_case(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone(), expand_form(&l.elements[1])];
    for clause in &l.elements[2..] {
        elems.push(expand_clause(clause));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_clause(clause: &SExp) -> SExp {
    match clause {
        SExp::List(l) if l.elements.len() >= 2 => {
            let pattern = &l.elements[0];
            let mut elems = vec![pattern.clone()];
            for body in &l.elements[1..] {
                elems.push(expand_form(body));
            }
            SExp::List(List::new(elems, l.pos))
        }
        other => other.clone(),
    }
}

fn expand_let(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone()];
    if let SExp::List(bindings) = &l.elements[1] {
        let expanded_bindings: Vec<SExp> = bindings
            .elements
            .iter()
            .map(|b| match b {
                SExp::List(pair) if pair.elements.len() == 2 => SExp::List(List::new(
                    vec![pair.elements[0].clone(), expand_form(&pair.elements[1])],
                    pair.pos,
                )),
                other => other.clone(),
            })
            .collect();
        elems.push(SExp::List(List::new(expanded_bindings, bindings.pos)));
    } else {
        elems.push(l.elements[1].clone());
    }
    for body in &l.elements[2..] {
        elems.push(expand_form(body));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_lambda(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone(), l.elements[1].clone()];
    for body in &l.elements[2..] {
        elems.push(expand_form(body));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_match_lambda(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone()];
    for clause in &l.elements[1..] {
        elems.push(expand_clause(clause));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_progn(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone()];
    for body in &l.elements[1..] {
        elems.push(expand_form(body));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_if(l: &List) -> SExp {
    let elems: Vec<SExp> = l
        .elements
        .iter()
        .enumerate()
        .map(|(i, e)| if i == 0 { e.clone() } else { expand_form(e) })
        .collect();
    SExp::List(List::new(elems, l.pos))
}

fn expand_try(l: &List) -> SExp {
    let elems: Vec<SExp> = l
        .elements
        .iter()
        .enumerate()
        .map(|(i, e)| if i == 0 { e.clone() } else { expand_form(e) })
        .collect();
    SExp::List(List::new(elems, l.pos))
}

fn expand_receive(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone()];
    for part in &l.elements[1..] {
        elems.push(expand_form(part));
    }
    SExp::List(List::new(elems, l.pos))
}

fn expand_comprehension(l: &List) -> SExp {
    let elems: Vec<SExp> = l
        .elements
        .iter()
        .enumerate()
        .map(|(i, e)| if i == 0 { e.clone() } else { expand_form(e) })
        .collect();
    SExp::List(List::new(elems, l.pos))
}

// ============================================================
// P-1: c*r macros → nested car/cdr
// ============================================================

// ============================================================
// P-2: let* → nested let
// ============================================================

fn expand_let_star(l: &List) -> SExp {
    if let SExp::List(bindings) = &l.elements[1] {
        let body: Vec<SExp> = l.elements[2..].iter().map(expand_form).collect();
        let mut progn = vec![sym("progn")];
        progn.extend(body);
        let mut result = SExp::List(List::new(progn, dp()));
        for binding in bindings.elements.iter().rev() {
            if let SExp::List(pair) = binding {
                if pair.elements.len() == 2 {
                    let expanded_binding = SExp::List(List::new(
                        vec![pair.elements[0].clone(), expand_form(&pair.elements[1])],
                        dp(),
                    ));
                    result = SExp::List(List::new(
                        vec![
                            sym("let"),
                            SExp::List(List::new(vec![expanded_binding], dp())),
                            result,
                        ],
                        dp(),
                    ));
                }
            }
        }
        result
    } else {
        expand_let(l)
    }
}

fn expand_cxr(name: &str, arg: &SExp) -> SExp {
    let ops: Vec<char> = name[1..name.len() - 1].chars().rev().collect();
    let mut result = expand_form(arg);
    for op in ops {
        let func = if op == 'a' { "car" } else { "cdr" };
        result = SExp::List(List::new(vec![sym(func), result], dp()));
    }
    result
}

// ============================================================
// P-2: list* → cons chain
// ============================================================

fn expand_list_star(args: &[SExp]) -> SExp {
    if args.len() == 1 {
        return expand_form(&args[0]);
    }
    let last = expand_form(&args[args.len() - 1]);
    let mut result = last;
    for arg in args[..args.len() - 1].iter().rev() {
        result = SExp::List(List::new(vec![sym("cons"), expand_form(arg), result], dp()));
    }
    result
}

// ============================================================
// P-2: flet* → nested let-function
// ============================================================

fn expand_flet_star(l: &List) -> SExp {
    if let SExp::List(bindings) = &l.elements[1] {
        let body: Vec<SExp> = l.elements[2..].iter().map(expand_form).collect();
        let mut progn = vec![sym("progn")];
        progn.extend(body);
        let mut result = SExp::List(List::new(progn, dp()));
        for binding in bindings.elements.iter().rev() {
            if let SExp::List(b) = binding {
                if b.elements.len() >= 2 {
                    let fname = b.elements[0].clone();
                    let fargs = &b.elements[1..];
                    let mut lambda_elems = Vec::new();
                    if fargs.len() >= 2 {
                        if let SExp::List(_) = &fargs[0] {
                            lambda_elems.push(sym("lambda"));
                            lambda_elems.push(fargs[0].clone());
                            for body_expr in &fargs[1..] {
                                lambda_elems.push(expand_form(body_expr));
                            }
                        }
                    }
                    let lambda = if lambda_elems.is_empty() {
                        expand_form(binding)
                    } else {
                        SExp::List(List::new(lambda_elems, dp()))
                    };
                    let fn_binding = SExp::List(List::new(vec![fname, lambda], dp()));
                    result = SExp::List(List::new(
                        vec![
                            sym("let-function"),
                            SExp::List(List::new(vec![fn_binding], dp())),
                            result,
                        ],
                        dp(),
                    ));
                }
            }
        }
        result
    } else {
        expand_form(&l.elements[1])
    }
}

// ============================================================
// P-2: ? → receive with timeout
// ============================================================

// ============================================================
// P-2: do → letrec-function with gensym loop name
// ============================================================

fn expand_do(l: &List) -> SExp {
    let loop_name = new_fun_name();
    let vars_form = &l.elements[1];
    let test_form = &l.elements[2];

    let mut var_names = Vec::new();
    let mut inits = Vec::new();
    let mut steps = Vec::new();

    if let SExp::List(vars) = vars_form {
        for var in &vars.elements {
            if let SExp::List(v) = var {
                if v.elements.len() >= 2 {
                    var_names.push(v.elements[0].clone());
                    inits.push(expand_form(&v.elements[1]));
                    if v.elements.len() >= 3 {
                        steps.push(expand_form(&v.elements[2]));
                    } else {
                        steps.push(v.elements[0].clone());
                    }
                }
            }
        }
    }

    let (test_expr, result_body) = if let SExp::List(test) = test_form {
        if test.elements.len() >= 2 {
            (
                expand_form(&test.elements[0]),
                expand_form(&test.elements[1]),
            )
        } else if test.elements.len() == 1 {
            (
                expand_form(&test.elements[0]),
                SExp::List(List::new(vec![sym("quote"), sym("true")], dp())),
            )
        } else {
            (sym("true"), sym("true"))
        }
    } else {
        (sym("true"), sym("true"))
    };

    let body_exprs: Vec<SExp> = l.elements[3..].iter().map(expand_form).collect();
    let do_state = if body_exprs.is_empty() {
        SExp::List(List::new(vec![sym("progn")], dp()))
    } else if body_exprs.len() == 1 {
        body_exprs.into_iter().next().unwrap()
    } else {
        let mut progn = vec![sym("progn")];
        progn.extend(body_exprs);
        SExp::List(List::new(progn, dp()))
    };

    let mut recurse_args = vec![sym(&loop_name)];
    recurse_args.extend(steps);
    let recurse = SExp::List(List::new(recurse_args, dp()));

    let if_form = SExp::List(List::new(
        vec![
            sym("if"),
            test_expr,
            result_body,
            SExp::List(List::new(
                vec![
                    sym("let"),
                    SExp::List(List::new(
                        vec![SExp::List(List::new(vec![sym("do-state"), do_state], dp()))],
                        dp(),
                    )),
                    recurse,
                ],
                dp(),
            )),
        ],
        dp(),
    ));

    let lambda_args = var_names.clone();
    let lambda = SExp::List(List::new(
        vec![
            sym("lambda"),
            SExp::List(List::new(lambda_args, dp())),
            if_form,
        ],
        dp(),
    ));

    let fn_binding = SExp::List(List::new(vec![sym(&loop_name), lambda], dp()));

    let mut call_args = vec![sym(&loop_name)];
    call_args.extend(inits);
    let call = SExp::List(List::new(call_args, dp()));

    SExp::List(List::new(
        vec![
            sym("letrec-function"),
            SExp::List(List::new(vec![fn_binding], dp())),
            call,
        ],
        dp(),
    ))
}

// ============================================================
// P-2: fun → lambda with gensym arg names
// ============================================================

fn expand_fun_ref(l: &List) -> SExp {
    let (mod_name, fun_name, arity) = if l.elements.len() == 4 {
        if let (SExp::Symbol(mod_s), SExp::Keyword(fun_k), SExp::Number(arity_n)) =
            (&l.elements[1], &l.elements[2], &l.elements[3])
        {
            (
                mod_s.value.clone(),
                fun_k.name.clone(),
                arity_n.value.parse::<usize>().ok(),
            )
        } else {
            return form_with_expanded_tail(l);
        }
    } else if l.elements.len() == 3 {
        if let (SExp::Symbol(mod_s), SExp::Number(arity_n)) = (&l.elements[1], &l.elements[2]) {
            if let Some((m, f)) = mod_s.value.split_once(':') {
                (
                    m.to_string(),
                    f.to_string(),
                    arity_n.value.parse::<usize>().ok(),
                )
            } else {
                return form_with_expanded_tail(l);
            }
        } else {
            return form_with_expanded_tail(l);
        }
    } else {
        return form_with_expanded_tail(l);
    };

    if let Some(arity) = arity {
        let mut args = Vec::new();
        for _ in 0..arity {
            args.push(sym(&new_var_name()));
        }
        let mut call_elems = vec![
            sym("call"),
            quoted(&sym(&mod_name)),
            quoted(&sym(&fun_name)),
        ];
        call_elems.extend(args.clone());
        SExp::List(List::new(
            vec![
                sym("lambda"),
                SExp::List(List::new(args, dp())),
                SExp::List(List::new(call_elems, dp())),
            ],
            dp(),
        ))
    } else {
        form_with_expanded_tail(l)
    }
}

fn expand_receive_timeout(l: &List) -> SExp {
    let timeout = expand_form(&l.elements[1]);
    let default = if l.elements.len() >= 3 {
        expand_form(&l.elements[2])
    } else {
        SExp::List(List::new(vec![sym("quote"), sym("true")], dp()))
    };
    SExp::List(List::new(
        vec![
            sym("receive"),
            SExp::List(List::new(vec![sym("omega"), sym("omega")], dp())),
            SExp::List(List::new(vec![sym("after"), timeout, default], dp())),
        ],
        dp(),
    ))
}

// ============================================================
// P-4: flet → let-function
// ============================================================

fn expand_flet(l: &List) -> SExp {
    if let SExp::List(bindings) = &l.elements[1] {
        let expanded_bindings: Vec<SExp> = bindings
            .elements
            .iter()
            .map(|b| {
                if let SExp::List(bd) = b {
                    if bd.elements.len() >= 2 {
                        let fname = bd.elements[0].clone();
                        let fargs = &bd.elements[1..];
                        if let SExp::List(_) = &fargs[0] {
                            let mut lambda_elems = vec![sym("lambda"), fargs[0].clone()];
                            for body_expr in &fargs[1..] {
                                lambda_elems.push(expand_form(body_expr));
                            }
                            return SExp::List(List::new(
                                vec![fname, SExp::List(List::new(lambda_elems, dp()))],
                                dp(),
                            ));
                        }
                    }
                }
                b.clone()
            })
            .collect();
        let mut elems = vec![
            sym("let-function"),
            SExp::List(List::new(expanded_bindings, dp())),
        ];
        for body in &l.elements[2..] {
            elems.push(expand_form(body));
        }
        SExp::List(List::new(elems, dp()))
    } else {
        form_with_expanded_tail(l)
    }
}

// ============================================================
// P-4: fletrec → letrec-function
// ============================================================

fn expand_fletrec(l: &List) -> SExp {
    if let SExp::List(bindings) = &l.elements[1] {
        let expanded_bindings: Vec<SExp> = bindings
            .elements
            .iter()
            .map(|b| {
                if let SExp::List(bd) = b {
                    if bd.elements.len() >= 2 {
                        let fname = bd.elements[0].clone();
                        let fargs = &bd.elements[1..];
                        if let SExp::List(_) = &fargs[0] {
                            let mut lambda_elems = vec![sym("lambda"), fargs[0].clone()];
                            for body_expr in &fargs[1..] {
                                lambda_elems.push(expand_form(body_expr));
                            }
                            return SExp::List(List::new(
                                vec![fname, SExp::List(List::new(lambda_elems, dp()))],
                                dp(),
                            ));
                        }
                    }
                }
                b.clone()
            })
            .collect();
        let mut elems = vec![
            sym("letrec-function"),
            SExp::List(List::new(expanded_bindings, dp())),
        ];
        for body in &l.elements[2..] {
            elems.push(expand_form(body));
        }
        SExp::List(List::new(elems, dp()))
    } else {
        form_with_expanded_tail(l)
    }
}

// ============================================================
// P-4: macrolet — expand the body (Tier-1: inline expansion)
// ============================================================

fn expand_macrolet(l: &List) -> SExp {
    let mut elems = vec![sym("progn")];
    for body in &l.elements[2..] {
        elems.push(expand_form(body));
    }
    if elems.len() == 2 {
        elems.pop().unwrap()
    } else {
        SExp::List(List::new(elems, dp()))
    }
}

// ============================================================
// P-4: deftype → define-type
// ============================================================

fn expand_deftype(l: &List) -> SExp {
    let name = &l.elements[1];
    let name_form = match name {
        SExp::List(_) => name.clone(),
        _ => SExp::List(List::new(vec![name.clone()], dp())),
    };
    let mut elems = vec![sym("define-type"), name_form];
    if l.elements.len() > 2 {
        elems.push(SExp::List(List::new(vec![], dp())));
    }
    SExp::List(List::new(elems, l.pos))
}

// ============================================================
// P-4: defspec → define-function-spec
// ============================================================

fn expand_defspec(l: &List) -> SExp {
    let name = &l.elements[1];
    let specs = &l.elements[2..];
    let mut spec_elems = Vec::new();
    for s in specs {
        spec_elems.push(s.clone());
    }
    SExp::List(List::new(
        vec![
            sym("define-function-spec"),
            SExp::List(List::new(
                vec![name.clone(), SExp::Number(Number::new("1", dp()))],
                dp(),
            )),
            SExp::List(List::new(spec_elems, dp())),
        ],
        l.pos,
    ))
}

fn form_with_expanded_tail(l: &List) -> SExp {
    let elems: Vec<SExp> = l
        .elements
        .iter()
        .enumerate()
        .map(|(i, e)| if i == 0 { e.clone() } else { expand_form(e) })
        .collect();
    SExp::List(List::new(elems, l.pos))
}

// ============================================================
// CL def* lowering
// ============================================================

fn expand_defun(l: &List) -> SExp {
    let name = l.elements[1].clone();
    if let SExp::List(args) = &l.elements[2] {
        if args.elements.iter().all(|e| matches!(e, SExp::Symbol(_))) {
            let mut lambda_elems = vec![sym("lambda"), l.elements[2].clone()];
            for body in &l.elements[3..] {
                lambda_elems.push(expand_form(body));
            }
            return SExp::List(List::new(
                vec![
                    sym("define-function"),
                    name,
                    SExp::List(List::new(vec![], dp())),
                    SExp::List(List::new(lambda_elems, dp())),
                ],
                l.pos,
            ));
        }
    }
    let mut clauses = Vec::new();
    for clause_form in &l.elements[2..] {
        if let SExp::List(_) = clause_form {
            let expanded_clause = expand_clause(clause_form);
            clauses.push(expanded_clause);
        } else {
            clauses.push(clause_form.clone());
        }
    }
    let mut ml_elems = vec![sym("match-lambda")];
    ml_elems.extend(clauses);
    SExp::List(List::new(
        vec![
            sym("define-function"),
            name,
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(ml_elems, dp())),
        ],
        l.pos,
    ))
}

fn expand_defmodule(l: &List) -> SExp {
    let name = &l.elements[1];
    let mut attrs = Vec::new();
    for attr in &l.elements[2..] {
        attrs.push(expand_form(attr));
    }
    let module_name_str = match name {
        SExp::Symbol(s) => s.value.clone(),
        _ => "unknown".to_string(),
    };
    set_module_name(&module_name_str);

    let define_module = SExp::List(List::new(
        vec![
            sym("define-module"),
            name.clone(),
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(attrs, dp())),
        ],
        l.pos,
    ));

    let module_macro = SExp::List(List::new(
        vec![
            sym("define-macro"),
            sym("MODULE"),
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(
                vec![
                    sym("match-lambda"),
                    SExp::List(List::new(
                        vec![
                            SExp::List(List::new(
                                vec![SExp::List(List::new(vec![sym("list")], dp())), sym("$ENV")],
                                dp(),
                            )),
                            SExp::List(List::new(
                                vec![
                                    sym("backquote"),
                                    SExp::List(List::new(
                                        vec![sym("quote"), sym(&module_name_str)],
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

    SExp::List(List::new(
        vec![sym("progn"), define_module, module_macro],
        l.pos,
    ))
}

fn expand_defmacro(l: &List) -> SExp {
    let name = l.elements[1].clone();
    if l.elements.len() >= 4 {
        if let SExp::List(args_list) = &l.elements[2] {
            if args_list
                .elements
                .iter()
                .all(|e| matches!(e, SExp::Symbol(_)))
            {
                let arg_pattern = vec![
                    SExp::List(List::new(
                        {
                            let mut a = vec![sym("list")];
                            a.extend(args_list.elements.clone());
                            a
                        },
                        dp(),
                    )),
                    sym("$ENV"),
                ];
                let mut clause_elems = vec![SExp::List(List::new(arg_pattern, dp()))];
                for body in &l.elements[3..] {
                    clause_elems.push(expand_form(body));
                }
                let clause = SExp::List(List::new(clause_elems, dp()));
                return SExp::List(List::new(
                    vec![
                        sym("define-macro"),
                        name,
                        SExp::List(List::new(vec![], dp())),
                        SExp::List(List::new(vec![sym("match-lambda"), clause], dp())),
                    ],
                    l.pos,
                ));
            }
        }
    }
    let mut clauses = Vec::new();
    for clause_form in &l.elements[2..] {
        if let SExp::List(clause) = clause_form {
            let mut clause_elems = Vec::new();
            if let Some(first) = clause.elements.first() {
                let args = vec![first.clone(), sym("$ENV")];
                clause_elems.push(SExp::List(List::new(args, dp())));
            }
            for body in clause.elements.iter().skip(1) {
                clause_elems.push(expand_form(body));
            }
            clauses.push(SExp::List(List::new(clause_elems, dp())));
        } else {
            clauses.push(clause_form.clone());
        }
    }
    let mut ml_elems = vec![sym("match-lambda")];
    ml_elems.extend(clauses);
    SExp::List(List::new(
        vec![
            sym("define-macro"),
            name,
            SExp::List(List::new(vec![], dp())),
            SExp::List(List::new(ml_elems, dp())),
        ],
        l.pos,
    ))
}

fn expand_cond(l: &List) -> SExp {
    let clauses = &l.elements[1..];
    expand_cond_clauses(clauses)
}

fn expand_cond_clauses(clauses: &[SExp]) -> SExp {
    if clauses.is_empty() {
        return SExp::List(List::new(vec![sym("quote"), sym("false")], dp()));
    }
    match &clauses[0] {
        SExp::List(clause) if !clause.elements.is_empty() => {
            let test = expand_form(&clause.elements[0]);
            let body: Vec<SExp> = clause.elements[1..].iter().map(expand_form).collect();
            let else_branch = expand_cond_clauses(&clauses[1..]);
            let mut if_elems = vec![sym("if"), test];
            if body.len() == 1 {
                if_elems.push(body.into_iter().next().unwrap());
            } else {
                let mut progn = vec![sym("progn")];
                progn.extend(body);
                if_elems.push(SExp::List(List::new(progn, dp())));
            }
            if_elems.push(else_branch);
            SExp::List(List::new(if_elems, dp()))
        }
        _ => SExp::List(List::new(vec![sym("quote"), sym("false")], dp())),
    }
}

fn expand_let_function(l: &List) -> SExp {
    let mut elems = vec![l.elements[0].clone()];
    if l.elements.len() >= 2 {
        if let SExp::List(fns) = &l.elements[1] {
            let expanded_fns: Vec<SExp> = fns
                .elements
                .iter()
                .map(|f| match f {
                    SExp::List(fd) if fd.elements.len() >= 2 => {
                        let mut fn_elems = vec![fd.elements[0].clone()];
                        for part in &fd.elements[1..] {
                            fn_elems.push(expand_form(part));
                        }
                        SExp::List(List::new(fn_elems, fd.pos))
                    }
                    other => other.clone(),
                })
                .collect();
            elems.push(SExp::List(List::new(expanded_fns, fns.pos)));
        } else {
            elems.push(l.elements[1].clone());
        }
    }
    for body in &l.elements[2..] {
        elems.push(expand_form(body));
    }
    SExp::List(List::new(elems, l.pos))
}
