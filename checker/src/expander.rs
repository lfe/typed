use crate::error::Position;
use crate::sexp::types::*;

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
        "quote" | "define-module" | "define-record" => form.clone(),
        "case" if l.elements.len() >= 3 => expand_case(l),
        "let" | "let*" if l.elements.len() >= 3 => expand_let(l),
        "lambda" if l.elements.len() >= 3 => expand_lambda(l),
        "match-lambda" if l.elements.len() >= 2 => expand_match_lambda(l),
        "progn" => expand_progn(l),
        "if" => expand_if(l),
        "try" => expand_try(l),
        "receive" => expand_receive(l),
        "lc" | "bc" => expand_comprehension(l),
        "let-function" | "letrec-function" => expand_let_function(l),
        _ => form.clone(),
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
