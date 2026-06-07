use crate::sexp::Parser;
use crate::sexp::types::*;
use crate::typed_surface;
use crate::lower;
use crate::eetf;

#[test]
fn f2_parse_fixture_line_col() {
    let input = r#";; Line 1
;; Line 2
(defmodule hello
  (export (greet 1)))

;; Line 6 - deliberately placed on line 7
(defun/typed greet
  :args ((name binary))
  :returns binary
  :body (list "Hello " name))
"#;
    let forms = Parser::parse_all_str(input).unwrap();
    assert!(forms.len() >= 2);

    let defmodule = &forms[0];
    match defmodule {
        SExp::List(l) => {
            assert_eq!(l.pos.line, 3);
            assert_eq!(l.pos.column, 1);
        }
        _ => panic!("expected list"),
    }

    let defun_typed = &forms[1];
    match defun_typed {
        SExp::List(l) => {
            assert_eq!(l.pos.line, 7);
            assert_eq!(l.pos.column, 1);
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn f3_parse_typed_surface() {
    let input = r#"(defun/typed greet
  :args ((name binary))
  :returns binary
  :body (list "Hello " name))"#;
    let form = Parser::parse_str(input).unwrap();
    let tf = typed_surface::extract_typed_fun(&form).unwrap();
    assert_eq!(tf.name, "greet");
    assert_eq!(tf.args.len(), 1);
    assert_eq!(tf.args[0].0, "name");
    assert_eq!(tf.args[0].1, "binary");
    assert_eq!(tf.returns, "binary");
    assert!(!tf.body.is_empty());
}

#[test]
fn f4_malformed_diagnostic_has_line_col() {
    let input = r#"(defun/typed oops
  :args ((x integer)))"#;
    let form = Parser::parse_str(input).unwrap();
    let err = typed_surface::extract_typed_fun(&form).unwrap_err();
    match err {
        crate::error::CheckError::Diagnostic { pos, message, .. } => {
            assert_eq!(pos.line, 1, "diagnostic line");
            assert_eq!(pos.column, 1, "diagnostic column");
            assert!(
                message.contains("missing") || message.contains("requires"),
                "message was: {}", message
            );
        }
    }
}

#[test]
fn f5_lower_typed_fun() {
    let input = r#"(defun/typed greet
  :args ((name binary))
  :returns binary
  :body (list "Hello " name))"#;
    let form = Parser::parse_str(input).unwrap();
    let tf = typed_surface::extract_typed_fun(&form).unwrap();
    let lowered = lower::lower_typed_fun(&tf);

    assert_eq!(lowered.line, 1);

    match &lowered.module_form {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 4);
            match &l.elements[0] {
                SExp::Symbol(s) => assert_eq!(s.value, "define-function"),
                _ => panic!("expected define-function symbol"),
            }
            match &l.elements[1] {
                SExp::Symbol(s) => assert_eq!(s.value, "greet"),
                _ => panic!("expected name"),
            }
            match &l.elements[3] {
                SExp::List(lambda) => {
                    assert!(lambda.elements.len() >= 3);
                    match &lambda.elements[0] {
                        SExp::Symbol(s) => assert_eq!(s.value, "lambda"),
                        _ => panic!("expected lambda"),
                    }
                    match &lambda.elements[1] {
                        SExp::List(args) => {
                            assert_eq!(args.elements.len(), 1);
                            match &args.elements[0] {
                                SExp::Symbol(s) => assert_eq!(s.value, "name"),
                                _ => panic!("expected arg name"),
                            }
                        }
                        _ => panic!("expected args list"),
                    }
                }
                _ => panic!("expected lambda list"),
            }
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn f5_lower_preserves_original_line() {
    let input = ";; padding\n;; more padding\n;; yet more\n(defun/typed add\n  :args ((x integer) (y integer))\n  :returns integer\n  :body (+ x y))";
    let forms = Parser::parse_all_str(input).unwrap();
    let tf = typed_surface::extract_typed_fun(&forms[0]).unwrap();
    let lowered = lower::lower_typed_fun(&tf);
    assert_eq!(lowered.line, 4, "original source line should be 4");
}

#[test]
fn f6_eetf_encodes() {
    let input = r#"(defun/typed greet
  :args ((name binary))
  :returns binary
  :body (list "Hello " name))"#;
    let form = Parser::parse_str(input).unwrap();
    let tf = typed_surface::extract_typed_fun(&form).unwrap();
    let lowered = lower::lower_typed_fun(&tf);

    let module_form = lower::lower_module_def("hello", &[("greet".to_string(), 1)]);

    let pairs = vec![
        (module_form, 1usize),
        (lowered.module_form, lowered.line),
    ];

    let bytes = eetf::encode_forms(&pairs);
    assert!(bytes.len() > 10);
    assert_eq!(bytes[0], 131);
}
