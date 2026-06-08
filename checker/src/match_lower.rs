use crate::adt::{AdtDef, CtorDef, ReprKind};
use crate::error::Position;
use crate::lower::to_snake_case;
use crate::matching::{MatchClause, Pattern, TypedMatch};
use crate::sexp::types::*;

pub fn lower_case_typed(typed_match: &TypedMatch, adt: &AdtDef, otp_version: u32) -> SExp {
    let repr = adt.effective_repr(otp_version);
    let scrutinee = strip_pos(&typed_match.scrutinee);

    let mut case_clauses = Vec::new();
    for clause in &typed_match.clauses {
        let lowered = lower_clause(clause, adt, &repr);
        case_clauses.push(lowered);
    }

    let mut case_elems = vec![sym("case"), scrutinee];
    case_elems.extend(case_clauses);
    SExp::List(List::new(case_elems, dp()))
}

fn lower_clause(clause: &MatchClause, adt: &AdtDef, repr: &ReprKind) -> SExp {
    let pattern = lower_pattern(&clause.pattern, adt, repr);
    let body: Vec<SExp> = clause.body.iter().map(strip_pos).collect();

    let mut clause_elems = vec![pattern];
    clause_elems.extend(body);
    SExp::List(List::new(clause_elems, dp()))
}

fn lower_pattern(pattern: &Pattern, adt: &AdtDef, repr: &ReprKind) -> SExp {
    match pattern {
        Pattern::Wildcard { .. } => sym("_"),
        Pattern::Variable { name, .. } => sym(name),
        Pattern::Constructor { name, bindings, .. } => {
            let ctor_def = adt.find_ctor(name);
            match repr {
                ReprKind::TaggedTuple | ReprKind::Default => {
                    lower_tagged_tuple_pattern(name, bindings, ctor_def)
                }
                ReprKind::Enum => lower_enum_pattern(name),
                ReprKind::Transparent => lower_transparent_pattern(bindings),
                ReprKind::NativeRecord => lower_native_record_pattern(name, bindings, ctor_def),
            }
        }
    }
}

fn lower_tagged_tuple_pattern(
    ctor_name: &str,
    bindings: &[String],
    ctor_def: Option<&CtorDef>,
) -> SExp {
    let tag = to_snake_case(ctor_name);
    if bindings.is_empty() && ctor_def.is_some_and(|c| c.fields.is_empty()) {
        return quoted_atom(&tag);
    }
    let mut elems = vec![sym("tuple"), quoted_atom(&tag)];
    for binding in bindings {
        elems.push(sym(binding));
    }
    SExp::List(List::new(elems, dp()))
}

fn lower_enum_pattern(ctor_name: &str) -> SExp {
    quoted_atom(&to_snake_case(ctor_name))
}

fn lower_transparent_pattern(bindings: &[String]) -> SExp {
    if let Some(first) = bindings.first() {
        sym(first)
    } else {
        sym("_")
    }
}

fn lower_native_record_pattern(
    ctor_name: &str,
    bindings: &[String],
    ctor_def: Option<&CtorDef>,
) -> SExp {
    let tag = to_snake_case(ctor_name);
    if bindings.is_empty() && ctor_def.is_some_and(|c| c.fields.is_empty()) {
        return quoted_atom(&tag);
    }
    let mut elems = vec![sym("match-record"), quoted_atom(&tag)];
    if let Some(cdef) = ctor_def {
        for (i, field) in cdef.fields.iter().enumerate() {
            let binding = bindings.get(i).map(|s| s.as_str()).unwrap_or("_");
            elems.push(quoted_atom(&field.name));
            elems.push(sym(binding));
        }
    }
    SExp::List(List::new(elems, dp()))
}

fn strip_pos(sexp: &SExp) -> SExp {
    match sexp {
        SExp::Symbol(s) => SExp::Symbol(Symbol::new(s.value.clone(), dp())),
        SExp::Keyword(k) => SExp::Keyword(Keyword::new(k.name.clone(), dp())),
        SExp::String(s) => SExp::String(StringLit::new(s.value.clone(), dp())),
        SExp::Number(n) => SExp::Number(Number::new(n.value.clone(), dp())),
        SExp::Nil(_) => SExp::Nil(Nil::new(dp())),
        SExp::List(l) => {
            let elems = l.elements.iter().map(strip_pos).collect();
            SExp::List(List::new(elems, dp()))
        }
        SExp::Tuple(t) => {
            let elems = t.elements.iter().map(strip_pos).collect();
            SExp::Tuple(Tuple::new(elems, dp()))
        }
    }
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
