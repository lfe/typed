use std::collections::HashMap;

use crate::adt::AdtDef;
use crate::error::{CheckError, Position};
use crate::sexp::types::*;
use crate::type_env::TypeEnv;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Float,
    Number,
    Atom,
    Boolean,
    Binary,
    String,
    List,
    Adt(String),
    Dynamic,
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer => write!(f, "integer"),
            Type::Float => write!(f, "float"),
            Type::Number => write!(f, "number"),
            Type::Atom => write!(f, "atom"),
            Type::Boolean => write!(f, "boolean"),
            Type::Binary => write!(f, "binary"),
            Type::String => write!(f, "string"),
            Type::List => write!(f, "list"),
            Type::Adt(name) => write!(f, "{}", name),
            Type::Dynamic => write!(f, "dynamic"),
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn parse_type(name: &str) -> Type {
    match name {
        "integer" => Type::Integer,
        "float" => Type::Float,
        "number" => Type::Number,
        "atom" => Type::Atom,
        "boolean" => Type::Boolean,
        "binary" => Type::Binary,
        "string" => Type::String,
        "list" => Type::List,
        "dynamic" => Type::Dynamic,
        other => Type::Adt(other.to_string()),
    }
}

pub fn types_compatible(got: &Type, expected: &Type) -> bool {
    if *got == Type::Dynamic || *expected == Type::Dynamic {
        return true;
    }
    if *got == Type::Unknown || *expected == Type::Unknown {
        return true;
    }
    if got == expected {
        return true;
    }
    matches!(
        (got, expected),
        (Type::Integer, Type::Number)
            | (Type::Float, Type::Number)
            | (Type::Number, Type::Integer)
            | (Type::Number, Type::Float)
            | (Type::String, Type::List)
            | (Type::List, Type::String)
    )
}

#[derive(Debug, Clone)]
pub struct FunSig {
    pub args: Vec<(String, Type)>,
    pub returns: Type,
}

pub struct BodyEnv {
    vars: HashMap<String, Type>,
    fun_sigs: HashMap<String, FunSig>,
}

impl BodyEnv {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            fun_sigs: HashMap::new(),
        }
    }

    pub fn bind_var(&mut self, name: &str, ty: Type) {
        self.vars.insert(name.to_string(), ty);
    }

    pub fn lookup_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    pub fn register_fun(&mut self, name: &str, sig: FunSig) {
        self.fun_sigs.insert(name.to_string(), sig);
    }

    pub fn lookup_fun(&self, name: &str) -> Option<&FunSig> {
        self.fun_sigs.get(name)
    }
}

fn builtin_return_type(op: &str, arg_count: usize) -> Option<Type> {
    match (op, arg_count) {
        ("+" | "-" | "*" | "div" | "rem", 2) => Some(Type::Number),
        ("+" | "-", 1) => Some(Type::Number),
        (">" | "<" | ">=" | "=<" | "==" | "/=" | "=:=" | "=/=", 2) => Some(Type::Boolean),
        ("and" | "or" | "not" | "xor" | "andalso" | "orelse", _) => Some(Type::Boolean),
        ("++" | "list", _) => Some(Type::List),
        ("tuple", _) => Some(Type::Dynamic),
        ("error", _) => Some(Type::Dynamic),
        ("length" | "size" | "tuple_size" | "byte_size" | "bit_size", 1) => Some(Type::Integer),
        ("hd" | "tl", 1) => Some(Type::Dynamic),
        (
            "is_integer" | "is_float" | "is_atom" | "is_binary" | "is_list" | "is_tuple"
            | "is_boolean" | "is_number",
            1,
        ) => Some(Type::Boolean),
        _ => None,
    }
}

pub fn synth_expr(
    expr: &SExp,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
) -> (Type, Vec<CheckError>) {
    let mut errors = Vec::new();

    let ty = match expr {
        SExp::Number(n) => {
            if n.value.contains('.') {
                Type::Float
            } else {
                Type::Integer
            }
        }
        SExp::String(_) => Type::String,
        SExp::Nil(_) => Type::List,
        SExp::Keyword(_) => Type::Atom,
        SExp::Symbol(s) => {
            if s.value == "true" || s.value == "false" {
                Type::Boolean
            } else if let Some(ty) = env.lookup_var(&s.value) {
                ty.clone()
            } else {
                Type::Dynamic
            }
        }
        SExp::List(l) => {
            if l.elements.is_empty() {
                return (Type::List, errors);
            }
            match &l.elements[0] {
                SExp::Symbol(s) if s.value == "quote" => synth_quote(l),
                SExp::Symbol(s) if s.value == "if" => synth_if(l, env, type_env, file, &mut errors),
                SExp::Symbol(s) if s.value == "let" || s.value == "let*" => {
                    synth_let(l, env, type_env, file, &mut errors)
                }
                SExp::Symbol(s)
                    if matches!(
                        s.value.as_str(),
                        "case" | "case/typed" | "tuple" | "backquote" | "comma" | "comma-at"
                    ) =>
                {
                    Type::Dynamic
                }
                SExp::Symbol(s) if s.value == "list" => Type::List,
                SExp::Symbol(s) if s.value == "binary" => Type::Binary,
                SExp::Symbol(s) => synth_call(l, &s.value, env, type_env, file, &mut errors),
                _ => Type::Dynamic,
            }
        }
    };

    (ty, errors)
}

fn synth_quote(l: &List) -> Type {
    if l.elements.len() == 2 {
        match &l.elements[1] {
            SExp::Symbol(_) => Type::Atom,
            SExp::Number(n) => {
                if n.value.contains('.') {
                    Type::Float
                } else {
                    Type::Integer
                }
            }
            SExp::String(_) => Type::String,
            SExp::List(_) => Type::List,
            _ => Type::Dynamic,
        }
    } else {
        Type::Dynamic
    }
}

fn synth_call(
    l: &List,
    func_name: &str,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    errors: &mut Vec<CheckError>,
) -> Type {
    let arg_count = l.elements.len() - 1;

    if let Some(sig) = env.lookup_fun(func_name) {
        check_call_args(l, sig, env, type_env, file, errors);
        sig.returns.clone()
    } else if let Some(ret) = builtin_return_type(func_name, arg_count) {
        ret
    } else if let Some((_rec_name, field_type)) = type_env.lookup_record_accessor(func_name) {
        parse_type(field_type)
    } else if let Some((rec_name, bad_field, available)) =
        type_env.check_unknown_record_field(func_name)
    {
        errors.push(CheckError::Diagnostic {
            file: file.to_string(),
            pos: l.pos,
            message: format!(
                "unknown field `{}` on record `{}`; available fields: {}",
                bad_field,
                rec_name,
                available.join(", ")
            ),
        });
        Type::Dynamic
    } else {
        Type::Dynamic
    }
}

fn synth_if(
    l: &List,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    errors: &mut Vec<CheckError>,
) -> Type {
    if l.elements.len() < 4 {
        return Type::Dynamic;
    }
    let (cond_ty, mut cond_errs) = synth_expr(&l.elements[1], env, type_env, file);
    errors.append(&mut cond_errs);
    if !types_compatible(&cond_ty, &Type::Boolean) {
        errors.push(CheckError::Diagnostic {
            file: file.to_string(),
            pos: l.elements[1].position(),
            message: format!("if condition must be boolean, got `{}`", cond_ty),
        });
    }
    let (then_ty, mut then_errs) = synth_expr(&l.elements[2], env, type_env, file);
    errors.append(&mut then_errs);
    let (_else_ty, mut else_errs) = synth_expr(&l.elements[3], env, type_env, file);
    errors.append(&mut else_errs);
    then_ty
}

fn synth_let(
    l: &List,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    errors: &mut Vec<CheckError>,
) -> Type {
    if l.elements.len() < 3 {
        return Type::Dynamic;
    }
    let mut inner_env = BodyEnv::new();
    for (k, v) in &env.vars {
        inner_env.bind_var(k, v.clone());
    }
    for (k, v) in &env.fun_sigs {
        inner_env.register_fun(k, v.clone());
    }

    if let SExp::List(bindings) = &l.elements[1] {
        for binding in &bindings.elements {
            if let SExp::List(pair) = binding {
                if pair.elements.len() == 2 {
                    if let SExp::Symbol(var_name) = &pair.elements[0] {
                        let (val_ty, mut val_errs) =
                            synth_expr(&pair.elements[1], &inner_env, type_env, file);
                        errors.append(&mut val_errs);
                        inner_env.bind_var(&var_name.value, val_ty);
                    }
                }
            }
        }
    }

    let mut last_ty = Type::Dynamic;
    for body_expr in &l.elements[2..] {
        let (ty, mut body_errs) = synth_expr(body_expr, &inner_env, type_env, file);
        errors.append(&mut body_errs);
        last_ty = ty;
    }
    last_ty
}

fn check_call_args(
    call: &List,
    sig: &FunSig,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    errors: &mut Vec<CheckError>,
) {
    let arg_exprs = &call.elements[1..];
    if arg_exprs.len() != sig.args.len() {
        errors.push(CheckError::Diagnostic {
            file: file.to_string(),
            pos: call.pos,
            message: format!(
                "function expects {} argument(s), got {}",
                sig.args.len(),
                arg_exprs.len()
            ),
        });
        return;
    }
    for (i, arg_expr) in arg_exprs.iter().enumerate() {
        let (arg_ty, mut arg_errs) = synth_expr(arg_expr, env, type_env, file);
        errors.append(&mut arg_errs);
        let expected = &sig.args[i].1;
        if !types_compatible(&arg_ty, expected) {
            errors.push(CheckError::Diagnostic {
                file: file.to_string(),
                pos: arg_expr.position(),
                message: format!(
                    "argument `{}` expected type `{}`, got `{}`",
                    sig.args[i].0, expected, arg_ty
                ),
            });
        }
    }
}

pub fn check_body_return(
    body: &[SExp],
    returns_type: &str,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    body_pos: Position,
) -> Vec<CheckError> {
    let mut errors = Vec::new();
    let expected = parse_type(returns_type);

    if body.is_empty() {
        return errors;
    }

    let last = &body[body.len() - 1];
    let (got, mut synth_errs) = synth_expr(last, env, type_env, file);
    errors.append(&mut synth_errs);

    if !types_compatible(&got, &expected) {
        errors.push(CheckError::Diagnostic {
            file: file.to_string(),
            pos: body_pos,
            message: format!(
                "body returns `{}`, but contract declares `:returns {}`",
                got, expected
            ),
        });
    }

    errors
}

pub fn check_constructor_field_values(
    ctor_name: &str,
    fields: &[(String, SExp)],
    adt: &AdtDef,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    pos: Position,
) -> Vec<CheckError> {
    let mut errors = Vec::new();

    let ctor_def = match adt.find_ctor(ctor_name) {
        Some(c) => c,
        None => return errors,
    };

    for (provided_name, value_expr) in fields {
        if provided_name.starts_with('_') {
            continue;
        }
        if let Some(field_def) = ctor_def.fields.iter().find(|f| f.name == *provided_name) {
            let field_type = parse_type(&field_def.type_expr);
            if field_type == Type::Dynamic || matches!(field_type, Type::Adt(_)) {
                continue;
            }
            let (value_type, mut val_errs) = synth_expr(value_expr, env, type_env, file);
            errors.append(&mut val_errs);
            if !types_compatible(&value_type, &field_type) {
                errors.push(CheckError::Diagnostic {
                    file: file.to_string(),
                    pos,
                    message: format!(
                        "field `{}` of constructor `{}` expects type `{}`, got `{}`",
                        provided_name, ctor_name, field_type, value_type
                    ),
                });
            }
        }
    }

    errors
}

// C-2: Branch-body typing for case/typed

pub fn check_case_typed_branches(
    clauses_body: &[SExp],
    expected: &Type,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
) -> Vec<CheckError> {
    let mut errors = Vec::new();

    for clause_body in clauses_body {
        let (got, mut errs) = synth_expr(clause_body, env, type_env, file);
        errors.append(&mut errs);
        if !types_compatible(&got, expected) {
            errors.push(CheckError::Diagnostic {
                file: file.to_string(),
                pos: clause_body.position(),
                message: format!(
                    "case/typed branch returns `{}`, but expected `{}`",
                    got, expected
                ),
            });
        }
    }

    errors
}

// C-3: case/typed as function body checks against :returns

pub fn check_body_with_case_typed(
    body: &[SExp],
    returns_type: &str,
    env: &BodyEnv,
    type_env: &TypeEnv,
    _adt_env: &crate::type_env::TypeEnv,
    file: &str,
    body_pos: Position,
) -> Vec<CheckError> {
    let expected = parse_type(returns_type);

    if body.is_empty() {
        return Vec::new();
    }

    let last = &body[body.len() - 1];

    if is_case_typed_form(last) {
        if let Ok(tm) = crate::matching::extract_case_typed(last) {
            let clause_bodies: Vec<SExp> = tm
                .clauses
                .iter()
                .filter_map(|c| c.body.last().cloned())
                .collect();
            return check_case_typed_branches(&clause_bodies, &expected, env, type_env, file);
        }
    }

    check_body_return(body, returns_type, env, type_env, file, body_pos)
}

fn is_case_typed_form(expr: &SExp) -> bool {
    matches!(expr, SExp::List(l)
        if !l.elements.is_empty()
            && matches!(&l.elements[0], SExp::Symbol(s) if s.value == "case/typed"))
}

// C-5: Basic polymorphic contracts (test-only for now)

#[cfg(test)]
pub struct PolyEnv {
    bindings: HashMap<String, Type>,
}

#[cfg(test)]
impl PolyEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind_or_check(&mut self, var: &str, ty: &Type) -> Result<(), (String, Type, Type)> {
        if let Some(existing) = self.bindings.get(var) {
            if !types_compatible(ty, existing) {
                return Err((var.to_string(), existing.clone(), ty.clone()));
            }
            Ok(())
        } else {
            self.bindings.insert(var.to_string(), ty.clone());
            Ok(())
        }
    }
}

#[cfg(test)]
pub fn is_type_var(name: &str) -> bool {
    name.len() == 1 && name.chars().next().is_some_and(|c| c.is_lowercase())
}

#[cfg(test)]
pub fn check_poly_contract(
    args: &[(String, String)],
    arg_values: &[SExp],
    _returns: &str,
    env: &BodyEnv,
    type_env: &TypeEnv,
    file: &str,
    _pos: Position,
) -> Vec<CheckError> {
    let mut errors = Vec::new();
    let mut poly_env = PolyEnv::new();

    for (i, (_param_name, param_type_str)) in args.iter().enumerate() {
        if !is_type_var(param_type_str) {
            continue;
        }
        if let Some(arg_expr) = arg_values.get(i) {
            let (arg_ty, mut errs) = synth_expr(arg_expr, env, type_env, file);
            errors.append(&mut errs);
            if let Err((var, expected, got)) = poly_env.bind_or_check(param_type_str, &arg_ty) {
                errors.push(CheckError::Diagnostic {
                    file: file.to_string(),
                    pos: arg_expr.position(),
                    message: format!(
                        "type variable `{}` bound to `{}` by argument `{}`, but got `{}` here",
                        var,
                        expected,
                        args.iter()
                            .find(|(_, t)| *t == var)
                            .map(|(n, _)| n.as_str())
                            .unwrap_or("?"),
                        got
                    ),
                });
            }
        }
    }

    errors
}
