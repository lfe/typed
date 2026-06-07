use crate::error::{CheckError, Position};
use crate::sexp::types::*;

#[derive(Debug, Clone)]
pub struct TypedFun {
    pub name: String,
    pub args: Vec<(String, String)>,
    pub returns: String,
    pub body: Vec<SExp>,
    pub pos: Position,
}

#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub name: String,
    pub exports: Vec<(String, usize)>,
    #[expect(dead_code, reason = "will be used for diagnostics in M1")]
    pub pos: Position,
}

pub fn extract_module_def(form: &SExp) -> Option<ModuleDef> {
    let list = match form {
        SExp::List(l) => l,
        _ => return None,
    };
    let elems = &list.elements;
    if elems.is_empty() {
        return None;
    }
    let head = match &elems[0] {
        SExp::Symbol(s) if s.value == "defmodule" => s,
        _ => return None,
    };
    let name = match elems.get(1) {
        Some(SExp::Symbol(s)) => s.value.clone(),
        _ => return None,
    };
    let mut exports = Vec::new();
    for elem in &elems[2..] {
        if let SExp::List(attr_list) = elem {
            if let Some(SExp::Symbol(s)) = attr_list.elements.first() {
                if s.value == "export" {
                    for exp in &attr_list.elements[1..] {
                        if let SExp::List(pair) = exp {
                            if pair.elements.len() == 2 {
                                if let (Some(SExp::Symbol(fname)), Some(SExp::Number(arity))) =
                                    (pair.elements.first(), pair.elements.get(1))
                                {
                                    if let Ok(a) = arity.value.parse::<usize>() {
                                        exports.push((fname.value.clone(), a));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Some(ModuleDef {
        name,
        exports,
        pos: head.pos,
    })
}

pub fn extract_typed_fun(form: &SExp) -> Result<TypedFun, CheckError> {
    let list = match form {
        SExp::List(l) => l,
        _ => {
            return Err(CheckError::Diagnostic {
                file: String::new(),
                pos: form.position(),
                message: "expected a list form".to_string(),
            });
        }
    };

    let elems = &list.elements;
    if elems.is_empty() {
        return Err(CheckError::Diagnostic {
            file: String::new(),
            pos: list.pos,
            message: "empty form".to_string(),
        });
    }

    match &elems[0] {
        SExp::Symbol(s) if s.value == "defun/typed" => {}
        _ => {
            return Err(CheckError::Diagnostic {
                file: String::new(),
                pos: elems[0].position(),
                message: "expected defun/typed".to_string(),
            });
        }
    }

    if elems.len() < 5 {
        return Err(CheckError::Diagnostic {
            file: String::new(),
            pos: list.pos,
            message: format!(
                "defun/typed requires name, :args, :returns, and :body sections (got {} elements)",
                elems.len() - 1
            ),
        });
    }

    let name = match &elems[1] {
        SExp::Symbol(s) => s.value.clone(),
        other => {
            return Err(CheckError::Diagnostic {
                file: String::new(),
                pos: other.position(),
                message: "expected function name (symbol)".to_string(),
            });
        }
    };

    let mut args = Vec::new();
    let mut returns = String::new();
    let mut body = Vec::new();
    let mut saw_args = false;
    let mut saw_returns = false;
    let mut saw_body = false;

    let mut i = 2;
    while i < elems.len() {
        match &elems[i] {
            SExp::Keyword(k) if k.name == "args" => {
                saw_args = true;
                i += 1;
                if i >= elems.len() {
                    return Err(CheckError::Diagnostic {
                        file: String::new(),
                        pos: k.pos,
                        message: ":args requires a list of (name type) pairs".to_string(),
                    });
                }
                match &elems[i] {
                    SExp::List(arg_list) => {
                        for pair in &arg_list.elements {
                            match pair {
                                SExp::List(p) if p.elements.len() == 2 => {
                                    let arg_name = match &p.elements[0] {
                                        SExp::Symbol(s) => s.value.clone(),
                                        other => {
                                            return Err(CheckError::Diagnostic {
                                                file: String::new(),
                                                pos: other.position(),
                                                message: "expected argument name".to_string(),
                                            });
                                        }
                                    };
                                    let arg_type = match &p.elements[1] {
                                        SExp::Symbol(s) => s.value.clone(),
                                        SExp::List(l) => format_sexp_flat(&SExp::List(l.clone())),
                                        other => {
                                            return Err(CheckError::Diagnostic {
                                                file: String::new(),
                                                pos: other.position(),
                                                message: "expected type expression".to_string(),
                                            });
                                        }
                                    };
                                    args.push((arg_name, arg_type));
                                }
                                other => {
                                    return Err(CheckError::Diagnostic {
                                        file: String::new(),
                                        pos: other.position(),
                                        message: "expected (name type) pair in :args".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    other => {
                        return Err(CheckError::Diagnostic {
                            file: String::new(),
                            pos: other.position(),
                            message: ":args value must be a list".to_string(),
                        });
                    }
                }
            }
            SExp::Keyword(k) if k.name == "returns" => {
                saw_returns = true;
                i += 1;
                if i >= elems.len() {
                    return Err(CheckError::Diagnostic {
                        file: String::new(),
                        pos: k.pos,
                        message: ":returns requires a type".to_string(),
                    });
                }
                returns = match &elems[i] {
                    SExp::Symbol(s) => s.value.clone(),
                    SExp::List(l) => format_sexp_flat(&SExp::List(l.clone())),
                    other => {
                        return Err(CheckError::Diagnostic {
                            file: String::new(),
                            pos: other.position(),
                            message: "expected type expression after :returns".to_string(),
                        });
                    }
                };
            }
            SExp::Keyword(k) if k.name == "body" => {
                saw_body = true;
                i += 1;
                while i < elems.len() {
                    body.push(elems[i].clone());
                    i += 1;
                }
                break;
            }
            other => {
                return Err(CheckError::Diagnostic {
                    file: String::new(),
                    pos: other.position(),
                    message:
                        "unexpected element in defun/typed; expected :args, :returns, or :body"
                            .to_string(),
                });
            }
        }
        i += 1;
    }

    if !saw_args {
        return Err(CheckError::Diagnostic {
            file: String::new(),
            pos: list.pos,
            message: "defun/typed missing :args section".to_string(),
        });
    }
    if !saw_returns {
        return Err(CheckError::Diagnostic {
            file: String::new(),
            pos: list.pos,
            message: "defun/typed missing :returns section".to_string(),
        });
    }
    if !saw_body {
        return Err(CheckError::Diagnostic {
            file: String::new(),
            pos: list.pos,
            message: "defun/typed missing :body section".to_string(),
        });
    }

    Ok(TypedFun {
        name,
        args,
        returns,
        body,
        pos: list.pos,
    })
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
    }
}
