use crate::sexp::types::*;
use crate::typed_surface::TypedFun;
use crate::error::Position;

#[derive(Debug)]
pub struct LoweredForm {
    pub module_form: SExp,
    pub line: usize,
}

pub fn lower_typed_fun(tf: &TypedFun) -> LoweredForm {
    let arg_names: Vec<SExp> = tf
        .args
        .iter()
        .map(|(name, _)| SExp::Symbol(Symbol::new(name.clone(), dummy_pos())))
        .collect();

    let mut body_exprs: Vec<SExp> = Vec::new();
    for expr in &tf.body {
        body_exprs.push(strip_positions(expr));
    }

    let mut lambda_elems = vec![
        SExp::Symbol(Symbol::new("lambda", dummy_pos())),
        SExp::List(List::new(arg_names, dummy_pos())),
    ];
    lambda_elems.extend(body_exprs);

    let define_function = SExp::List(List::new(
        vec![
            SExp::Symbol(Symbol::new("define-function", dummy_pos())),
            SExp::Symbol(Symbol::new(tf.name.clone(), dummy_pos())),
            SExp::List(List::new(vec![], dummy_pos())),
            SExp::List(List::new(lambda_elems, dummy_pos())),
        ],
        dummy_pos(),
    ));

    LoweredForm {
        module_form: define_function,
        line: tf.pos.line,
    }
}

pub fn lower_module_def(name: &str, exports: &[(String, usize)]) -> SExp {
    let mut export_pairs = Vec::new();
    for (fname, arity) in exports {
        export_pairs.push(SExp::List(List::new(
            vec![
                SExp::Symbol(Symbol::new(fname.clone(), dummy_pos())),
                SExp::Number(Number::new(arity.to_string(), dummy_pos())),
            ],
            dummy_pos(),
        )));
    }

    let export_attr = SExp::List(List::new(
        vec![
            vec![SExp::Symbol(Symbol::new("export", dummy_pos()))],
            export_pairs,
        ].concat(),
        dummy_pos(),
    ));

    let attrs = SExp::List(List::new(vec![export_attr], dummy_pos()));
    let metas = SExp::List(List::new(vec![], dummy_pos()));

    SExp::List(List::new(
        vec![
            SExp::Symbol(Symbol::new("define-module", dummy_pos())),
            SExp::Symbol(Symbol::new(name, dummy_pos())),
            metas,
            attrs,
        ],
        dummy_pos(),
    ))
}

fn strip_positions(sexp: &SExp) -> SExp {
    match sexp {
        SExp::Symbol(s) => SExp::Symbol(Symbol::new(s.value.clone(), dummy_pos())),
        SExp::Keyword(k) => SExp::Keyword(Keyword::new(k.name.clone(), dummy_pos())),
        SExp::String(s) => SExp::String(StringLit::new(s.value.clone(), dummy_pos())),
        SExp::Number(n) => SExp::Number(Number::new(n.value.clone(), dummy_pos())),
        SExp::Nil(_) => SExp::Nil(Nil::new(dummy_pos())),
        SExp::List(l) => {
            let elems = l.elements.iter().map(strip_positions).collect();
            SExp::List(List::new(elems, dummy_pos()))
        }
    }
}

fn dummy_pos() -> Position {
    Position::new(0, 0, 0)
}
