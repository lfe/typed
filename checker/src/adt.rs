use crate::error::{CheckError, Position};
use crate::sexp::types::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AdtDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub constructors: Vec<CtorDef>,
    pub repr: ReprKind,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CtorDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub type_expr: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReprKind {
    TaggedTuple,
    Enum,
    Transparent,
    NativeRecord,
    Default,
}

#[derive(Debug, Clone)]
pub struct Construction {
    pub ctor_name: String,
    pub fields: Vec<(String, SExp)>,
    pub pos: Position,
}

impl AdtDef {
    pub fn is_all_nullary(&self) -> bool {
        self.constructors.iter().all(|c| c.fields.is_empty())
    }

    pub fn is_newtype(&self) -> bool {
        self.constructors.len() == 1 && self.constructors[0].fields.len() == 1
    }

    pub fn effective_repr(&self, otp_version: u32) -> ReprKind {
        match &self.repr {
            ReprKind::Default => {
                if self.is_all_nullary() {
                    ReprKind::Enum
                } else if self.is_newtype() {
                    ReprKind::Transparent
                } else if otp_version >= 29 {
                    ReprKind::NativeRecord
                } else {
                    ReprKind::TaggedTuple
                }
            }
            other => other.clone(),
        }
    }

    pub fn find_ctor(&self, name: &str) -> Option<&CtorDef> {
        self.constructors.iter().find(|c| c.name == name)
    }
}

pub fn extract_deftype(form: &SExp) -> Result<AdtDef, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => {
            return Err(diagnostic(
                form.position(),
                "expected a list form for deftype/typed",
            ))
        }
    };

    let elems = &list.elements;
    if elems.len() < 3 {
        return Err(diagnostic(
            list.pos,
            "deftype/typed requires at least a name/params and one constructor",
        ));
    }

    match &elems[0] {
        SExp::Symbol(s) if s.value == "deftype/typed" => {}
        _ => return Err(diagnostic(elems[0].position(), "expected deftype/typed")),
    }

    let (name, type_params) = parse_type_header(&elems[1])?;

    let mut constructors = Vec::new();
    let mut repr = ReprKind::Default;
    let mut i = 2;

    while i < elems.len() {
        match &elems[i] {
            SExp::List(l) if !l.elements.is_empty() => {
                if let SExp::Symbol(s) = &l.elements[0] {
                    if s.value == "repr" {
                        repr = parse_repr_clause(l)?;
                        i += 1;
                        continue;
                    }
                }
                constructors.push(parse_constructor(&elems[i])?);
            }
            _ => {
                return Err(diagnostic(
                    elems[i].position(),
                    "expected constructor definition (CtorName (field type) ...)",
                ))
            }
        }
        i += 1;
    }

    if constructors.is_empty() {
        return Err(diagnostic(
            list.pos,
            "deftype/typed requires at least one constructor",
        ));
    }

    Ok(AdtDef {
        name,
        type_params,
        constructors,
        repr,
        pos: list.pos,
    })
}

fn parse_type_header(sexp: &SExp) -> Result<(String, Vec<String>), CheckError> {
    match sexp {
        SExp::Symbol(s) => Ok((s.value.clone(), Vec::new())),
        SExp::List(l) => {
            if l.elements.is_empty() {
                return Err(diagnostic(l.pos, "empty type header"));
            }
            let name = match &l.elements[0] {
                SExp::Symbol(s) => s.value.clone(),
                other => return Err(diagnostic(other.position(), "expected type name")),
            };
            let params = l.elements[1..]
                .iter()
                .map(|e| match e {
                    SExp::Symbol(s) => Ok(s.value.clone()),
                    other => Err(diagnostic(other.position(), "expected type parameter name")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((name, params))
        }
        _ => Err(diagnostic(
            sexp.position(),
            "expected type name or (name params...)",
        )),
    }
}

fn parse_repr_clause(list: &List) -> Result<ReprKind, CheckError> {
    if list.elements.len() != 2 {
        return Err(diagnostic(list.pos, "repr clause: (repr <backend>)"));
    }
    match &list.elements[1] {
        SExp::Symbol(s) => match s.value.as_str() {
            "tagged-tuple" => Ok(ReprKind::TaggedTuple),
            "enum" => Ok(ReprKind::Enum),
            "transparent" => Ok(ReprKind::Transparent),
            "native-record" => Ok(ReprKind::NativeRecord),
            other => Err(diagnostic(
                s.pos,
                &format!("unknown repr backend: {other}; expected tagged-tuple, enum, transparent, or native-record"),
            )),
        },
        other => Err(diagnostic(other.position(), "repr value must be a symbol")),
    }
}

fn parse_constructor(form: &SExp) -> Result<CtorDef, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => {
            return Err(diagnostic(
                form.position(),
                "expected constructor definition",
            ))
        }
    };

    if list.elements.is_empty() {
        return Err(diagnostic(list.pos, "empty constructor definition"));
    }

    let name = match &list.elements[0] {
        SExp::Symbol(s) => s.value.clone(),
        other => return Err(diagnostic(other.position(), "expected constructor name")),
    };

    let mut fields = Vec::new();
    for elem in &list.elements[1..] {
        match elem {
            SExp::List(field_list) if field_list.elements.len() == 2 => {
                let field_name = match &field_list.elements[0] {
                    SExp::Symbol(s) => s.value.clone(),
                    other => return Err(diagnostic(other.position(), "expected field name")),
                };
                let field_type = match &field_list.elements[1] {
                    SExp::Symbol(s) => s.value.clone(),
                    SExp::List(l) => format_sexp_flat(&SExp::List(l.clone())),
                    other => return Err(diagnostic(other.position(), "expected field type")),
                };
                fields.push(FieldDef {
                    name: field_name,
                    type_expr: field_type,
                    pos: field_list.pos,
                });
            }
            other => {
                return Err(diagnostic(
                    other.position(),
                    "expected field definition (name type)",
                ))
            }
        }
    }

    Ok(CtorDef {
        name,
        fields,
        pos: list.pos,
    })
}

pub fn extract_defrecord(form: &SExp) -> Result<AdtDef, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => {
            return Err(diagnostic(
                form.position(),
                "expected a list form for defrecord/typed",
            ))
        }
    };

    let elems = &list.elements;
    if elems.len() < 3 {
        return Err(diagnostic(
            list.pos,
            "defrecord/typed requires a name and at least one field",
        ));
    }

    match &elems[0] {
        SExp::Symbol(s) if s.value == "defrecord/typed" => {}
        _ => return Err(diagnostic(elems[0].position(), "expected defrecord/typed")),
    }

    let name = match &elems[1] {
        SExp::Symbol(s) => s.value.clone(),
        other => return Err(diagnostic(other.position(), "expected record name")),
    };

    let mut fields = Vec::new();
    for elem in &elems[2..] {
        match elem {
            SExp::List(field_list) if field_list.elements.len() == 2 => {
                let field_name = match &field_list.elements[0] {
                    SExp::Symbol(s) => s.value.clone(),
                    other => return Err(diagnostic(other.position(), "expected field name")),
                };
                let field_type = match &field_list.elements[1] {
                    SExp::Symbol(s) => s.value.clone(),
                    SExp::List(l) => format_sexp_flat(&SExp::List(l.clone())),
                    other => return Err(diagnostic(other.position(), "expected field type")),
                };
                fields.push(FieldDef {
                    name: field_name,
                    type_expr: field_type,
                    pos: field_list.pos,
                });
            }
            other => {
                return Err(diagnostic(
                    other.position(),
                    "expected field definition (name type)",
                ))
            }
        }
    }

    Ok(AdtDef {
        name: name.clone(),
        type_params: Vec::new(),
        constructors: vec![CtorDef {
            name,
            fields,
            pos: list.pos,
        }],
        repr: ReprKind::TaggedTuple,
        pos: list.pos,
    })
}

pub fn extract_construction(
    form: &SExp,
    ctor_names: &[String],
) -> Option<Result<Construction, CheckError>> {
    let list = match form {
        SExp::List(l) => l,
        _ => return None,
    };
    if list.elements.is_empty() {
        return None;
    }
    let head_name = match &list.elements[0] {
        SExp::Symbol(s) => &s.value,
        _ => return None,
    };
    if !ctor_names.contains(head_name) {
        return None;
    }

    let ctor_name = head_name.clone();
    let mut fields = Vec::new();
    let mut i = 1;
    while i < list.elements.len() {
        match &list.elements[i] {
            SExp::Keyword(k) => {
                i += 1;
                if i >= list.elements.len() {
                    return Some(Err(diagnostic(
                        k.pos,
                        &format!("field :{} requires a value", k.name),
                    )));
                }
                fields.push((k.name.clone(), list.elements[i].clone()));
            }
            _ => {
                fields.push((format!("_{}", fields.len()), list.elements[i].clone()));
            }
        }
        i += 1;
    }

    Some(Ok(Construction {
        ctor_name,
        fields,
        pos: list.pos,
    }))
}

pub fn check_construction(
    construction: &Construction,
    adt: &AdtDef,
    file: &str,
) -> Result<(), CheckError> {
    let ctor = adt
        .find_ctor(&construction.ctor_name)
        .ok_or_else(|| CheckError::Diagnostic {
            file: file.to_string(),
            pos: construction.pos,
            message: format!(
                "unknown constructor `{}` for type `{}`; available: {}",
                construction.ctor_name,
                adt.name,
                adt.constructors
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })?;

    if ctor.fields.is_empty() && construction.fields.is_empty() {
        return Ok(());
    }

    if construction.fields.len() != ctor.fields.len() {
        return Err(CheckError::Diagnostic {
            file: file.to_string(),
            pos: construction.pos,
            message: format!(
                "constructor `{}` expects {} field(s) ({}), got {}",
                ctor.name,
                ctor.fields.len(),
                ctor.fields
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                construction.fields.len()
            ),
        });
    }

    for (provided_name, _) in &construction.fields {
        if provided_name.starts_with('_') {
            continue;
        }
        if !ctor.fields.iter().any(|f| f.name == *provided_name) {
            return Err(CheckError::Diagnostic {
                file: file.to_string(),
                pos: construction.pos,
                message: format!(
                    "unknown field `{}` for constructor `{}`; available: {}",
                    provided_name,
                    ctor.name,
                    ctor.fields
                        .iter()
                        .map(|f| f.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }
    }

    for field_def in &ctor.fields {
        if !construction
            .fields
            .iter()
            .any(|(n, _)| *n == field_def.name || n.starts_with('_'))
        {
            return Err(CheckError::Diagnostic {
                file: file.to_string(),
                pos: construction.pos,
                message: format!(
                    "missing field `{}` in constructor `{}`",
                    field_def.name, ctor.name
                ),
            });
        }
    }

    Ok(())
}

fn format_sexp_flat(sexp: &SExp) -> String {
    match sexp {
        SExp::Symbol(s) => s.value.clone(),
        SExp::Keyword(k) => format!(":{}", k.name),
        SExp::String(s) => format!("\"{}\"", s.value),
        SExp::Number(n) => n.value.clone(),
        SExp::Nil(_) => "nil".to_string(),
        SExp::List(l) => {
            let inner: Vec<String> = l.elements.iter().map(format_sexp_flat).collect();
            format!("({})", inner.join(" "))
        }
        SExp::Tuple(t) => {
            let inner: Vec<String> = t.elements.iter().map(format_sexp_flat).collect();
            format!("#({})", inner.join(" "))
        }
    }
}

fn diagnostic(pos: Position, message: &str) -> CheckError {
    CheckError::Diagnostic {
        file: String::new(),
        pos,
        message: message.to_string(),
    }
}
