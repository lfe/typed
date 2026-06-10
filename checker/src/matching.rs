use crate::adt::AdtDef;
use crate::error::{CheckError, Position};
use crate::sexp::types::*;

#[derive(Debug, Clone)]
pub struct TypedMatch {
    pub scrutinee: SExp,
    pub scrutinee_type: Option<String>,
    pub clauses: Vec<MatchClause>,
    pub pos: Position,
}

#[derive(Debug, Clone)]
pub struct MatchClause {
    pub pattern: Pattern,
    pub when_guard: Option<SExp>,
    pub body: Vec<SExp>,
    pub pos: Position,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Constructor {
        name: String,
        bindings: Vec<String>,
        pos: Position,
    },
    Wildcard {
        #[expect(dead_code, reason = "used for diagnostic spans in future")]
        pos: Position,
    },
    Variable {
        name: String,
        #[expect(dead_code, reason = "used for diagnostic spans in future")]
        pos: Position,
    },
}

impl Pattern {
    pub fn ctor_name(&self) -> Option<&str> {
        match self {
            Pattern::Constructor { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn is_catch_all(&self) -> bool {
        matches!(self, Pattern::Wildcard { .. } | Pattern::Variable { .. })
    }
}

pub fn extract_case_typed(form: &SExp) -> Result<TypedMatch, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => return Err(diag(form.position(), "expected a list form for case/typed")),
    };
    let elems = &list.elements;
    if elems.len() < 3 {
        return Err(diag(
            list.pos,
            "case/typed requires a scrutinee and at least one clause",
        ));
    }
    match &elems[0] {
        SExp::Symbol(s) if s.value == "case/typed" => {}
        _ => return Err(diag(elems[0].position(), "expected case/typed")),
    }

    let scrutinee = elems[1].clone();

    let mut scrutinee_type = None;
    let mut clause_start = 2;
    if let Some(SExp::Keyword(k)) = elems.get(2) {
        if k.name == "type" {
            if let Some(SExp::Symbol(t)) = elems.get(3) {
                scrutinee_type = Some(t.value.clone());
                clause_start = 4;
            }
        }
    }

    let mut clauses = Vec::new();
    for elem in &elems[clause_start..] {
        clauses.push(parse_clause(elem)?);
    }

    if clauses.is_empty() {
        return Err(diag(list.pos, "case/typed requires at least one clause"));
    }

    Ok(TypedMatch {
        scrutinee,
        scrutinee_type,
        clauses,
        pos: list.pos,
    })
}

fn parse_clause(form: &SExp) -> Result<MatchClause, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => return Err(diag(form.position(), "expected a clause (pattern body...)")),
    };
    if list.elements.is_empty() {
        return Err(diag(list.pos, "empty clause"));
    }

    let pattern = parse_pattern(&list.elements[0])?;

    let (when_guard, body) = if list.elements.len() >= 3 {
        if let SExp::List(when_list) = &list.elements[1] {
            if !when_list.elements.is_empty() {
                if let SExp::Symbol(s) = &when_list.elements[0] {
                    if s.value == "when" && when_list.elements.len() >= 2 {
                        (
                            Some(when_list.elements[1].clone()),
                            list.elements[2..].to_vec(),
                        )
                    } else {
                        (None, list.elements[1..].to_vec())
                    }
                } else {
                    (None, list.elements[1..].to_vec())
                }
            } else {
                (None, list.elements[1..].to_vec())
            }
        } else {
            (None, list.elements[1..].to_vec())
        }
    } else {
        (None, list.elements[1..].to_vec())
    };

    Ok(MatchClause {
        pattern,
        when_guard,
        body,
        pos: list.pos,
    })
}

fn parse_pattern(form: &SExp) -> Result<Pattern, CheckError> {
    match form {
        SExp::Symbol(s) if s.value == "_" => Ok(Pattern::Wildcard { pos: s.pos }),
        SExp::Symbol(s) => Ok(Pattern::Variable {
            name: s.value.clone(),
            pos: s.pos,
        }),
        SExp::List(l) => {
            if l.elements.is_empty() {
                return Err(diag(l.pos, "empty pattern"));
            }
            let name = match &l.elements[0] {
                SExp::Symbol(s) => s.value.clone(),
                other => {
                    return Err(diag(
                        other.position(),
                        "expected constructor name in pattern",
                    ))
                }
            };
            let bindings: Vec<String> = l.elements[1..]
                .iter()
                .map(|e| match e {
                    SExp::Symbol(s) => Ok(s.value.clone()),
                    other => Err(diag(
                        other.position(),
                        "expected variable binding in pattern",
                    )),
                })
                .collect::<Result<_, _>>()?;

            Ok(Pattern::Constructor {
                name,
                bindings,
                pos: l.pos,
            })
        }
        _ => Err(diag(form.position(), "expected a pattern")),
    }
}

pub fn check_exhaustiveness(typed_match: &TypedMatch, adt: &AdtDef, file: &str) -> Vec<CheckError> {
    let mut errors = Vec::new();

    let has_catch_all = typed_match.clauses.iter().any(|c| c.pattern.is_catch_all());
    if has_catch_all {
        return errors;
    }

    let covered: Vec<&str> = typed_match
        .clauses
        .iter()
        .filter_map(|c| c.pattern.ctor_name())
        .collect();

    let missing: Vec<&str> = adt
        .constructors
        .iter()
        .filter(|c| !covered.contains(&c.name.as_str()))
        .map(|c| c.name.as_str())
        .collect();

    if !missing.is_empty() {
        errors.push(CheckError::NonExhaustive {
            file: file.to_string(),
            pos: typed_match.pos,
            type_name: adt.name.clone(),
            missing: missing.iter().map(|s| s.to_string()).collect(),
        });
    }

    errors
}

pub fn check_pattern_wellformedness(
    typed_match: &TypedMatch,
    adt: &AdtDef,
    file: &str,
) -> Vec<CheckError> {
    let mut errors = Vec::new();

    for clause in &typed_match.clauses {
        if let Pattern::Constructor {
            name,
            bindings,
            pos,
        } = &clause.pattern
        {
            match adt.find_ctor(name) {
                None => {
                    errors.push(CheckError::Diagnostic {
                        file: file.to_string(),
                        pos: *pos,
                        message: format!(
                            "unknown constructor `{}` in pattern for type `{}`; available: {}",
                            name,
                            adt.name,
                            adt.constructors
                                .iter()
                                .map(|c| c.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    });
                }
                Some(ctor_def) => {
                    if bindings.len() != ctor_def.fields.len() {
                        errors.push(CheckError::Diagnostic {
                            file: file.to_string(),
                            pos: *pos,
                            message: format!(
                                "constructor `{}` has {} field(s) ({}), but pattern binds {}",
                                name,
                                ctor_def.fields.len(),
                                ctor_def
                                    .fields
                                    .iter()
                                    .map(|f| f.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                bindings.len()
                            ),
                        });
                    }
                }
            }
        }
    }

    errors
}

pub fn check_redundancy(typed_match: &TypedMatch, file: &str) -> Vec<CheckError> {
    let mut warnings = Vec::new();
    let mut seen_ctors: Vec<&str> = Vec::new();
    let mut seen_catch_all = false;

    for clause in &typed_match.clauses {
        if seen_catch_all {
            warnings.push(CheckError::Diagnostic {
                file: file.to_string(),
                pos: clause.pos,
                message: "unreachable clause: a catch-all pattern already covers all values"
                    .to_string(),
            });
            continue;
        }

        match &clause.pattern {
            Pattern::Wildcard { .. } | Pattern::Variable { .. } => {
                seen_catch_all = true;
            }
            Pattern::Constructor { name, pos, .. } => {
                if seen_ctors.contains(&name.as_str()) {
                    warnings.push(CheckError::Diagnostic {
                        file: file.to_string(),
                        pos: *pos,
                        message: format!(
                            "redundant clause: constructor `{}` is already matched above",
                            name
                        ),
                    });
                }
                seen_ctors.push(name);
            }
        }
    }

    warnings
}

fn diag(pos: Position, message: &str) -> CheckError {
    CheckError::Diagnostic {
        file: String::new(),
        pos,
        message: message.to_string(),
    }
}
