use crate::adt;
use crate::eetf;
use crate::lower;
use crate::sexp::types::*;
use crate::sexp::Parser;
use crate::type_env::TypeEnv;
use crate::typed_surface;

// ============================================================
// M0 tests (preserved)
// ============================================================

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
                "message was: {}",
                message
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

    let pairs = vec![(module_form, 1usize), (lowered.module_form, lowered.line)];

    let bytes = eetf::encode_forms(&pairs);
    assert!(bytes.len() > 10);
    assert_eq!(bytes[0], 131);
}

// ============================================================
// M1 tests — ADT parsing (M1-1)
// ============================================================

#[test]
fn m1_1_parse_parametric_deftype() {
    let input = r#"(deftype (result ok err)
  (Ok    (value ok))
  (Error (reason err)))"#;
    let form = Parser::parse_str(input).unwrap();
    let adt = adt::extract_deftype(&form).unwrap();

    assert_eq!(adt.name, "result");
    assert_eq!(adt.type_params, vec!["ok", "err"]);
    assert_eq!(adt.constructors.len(), 2);

    assert_eq!(adt.constructors[0].name, "Ok");
    assert_eq!(adt.constructors[0].fields.len(), 1);
    assert_eq!(adt.constructors[0].fields[0].name, "value");
    assert_eq!(adt.constructors[0].fields[0].type_expr, "ok");

    assert_eq!(adt.constructors[1].name, "Error");
    assert_eq!(adt.constructors[1].fields.len(), 1);
    assert_eq!(adt.constructors[1].fields[0].name, "reason");
    assert_eq!(adt.constructors[1].fields[0].type_expr, "err");
}

#[test]
fn m1_1_parse_nullary_deftype() {
    let input = "(deftype colour (Red) (Green) (Blue))";
    let form = Parser::parse_str(input).unwrap();
    let adt_def = adt::extract_deftype(&form).unwrap();

    assert_eq!(adt_def.name, "colour");
    assert!(adt_def.type_params.is_empty());
    assert_eq!(adt_def.constructors.len(), 3);
    assert!(adt_def.is_all_nullary());
    for ctor in &adt_def.constructors {
        assert!(ctor.fields.is_empty());
    }
}

#[test]
fn m1_1_parse_newtype_deftype() {
    let input = "(deftype customer-id (CustomerId (v integer)))";
    let form = Parser::parse_str(input).unwrap();
    let adt_def = adt::extract_deftype(&form).unwrap();

    assert_eq!(adt_def.name, "customer-id");
    assert!(adt_def.is_newtype());
    assert_eq!(adt_def.constructors[0].name, "CustomerId");
    assert_eq!(adt_def.constructors[0].fields[0].name, "v");
}

#[test]
fn m1_1_parse_deftype_with_repr() {
    let input = r#"(deftype (result ok err)
  (repr tagged-tuple)
  (Ok    (value ok))
  (Error (reason err)))"#;
    let form = Parser::parse_str(input).unwrap();
    let adt_def = adt::extract_deftype(&form).unwrap();

    assert_eq!(adt_def.repr, adt::ReprKind::TaggedTuple);
    assert_eq!(adt_def.constructors.len(), 2);
}

// ============================================================
// M1-2: Type environment
// ============================================================

#[test]
fn m1_2_type_env_register_and_lookup() {
    let input1 = "(deftype colour (Red) (Green) (Blue))";
    let input2 = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;

    let adt1 = adt::extract_deftype(&Parser::parse_str(input1).unwrap()).unwrap();
    let adt2 = adt::extract_deftype(&Parser::parse_str(input2).unwrap()).unwrap();

    let mut env = TypeEnv::new();
    env.register(adt1);
    env.register(adt2);

    assert!(env.lookup_type("colour").is_some());
    assert!(env.lookup_type("result").is_some());
    assert!(env.lookup_type("nonexistent").is_none());

    let by_ctor = env.lookup_ctor("Ok");
    assert!(by_ctor.is_some());
    assert_eq!(by_ctor.unwrap().name, "result");

    let by_ctor2 = env.lookup_ctor("Red");
    assert!(by_ctor2.is_some());
    assert_eq!(by_ctor2.unwrap().name, "colour");
}

// ============================================================
// M1-3: Construction parsing
// ============================================================

#[test]
fn m1_3_parse_construction() {
    let input = "(Ok :value 42)";
    let form = Parser::parse_str(input).unwrap();
    let ctor_names = vec!["Ok".to_string(), "Error".to_string()];
    let result = adt::extract_construction(&form, &ctor_names);
    assert!(result.is_some());
    let construction = result.unwrap().unwrap();
    assert_eq!(construction.ctor_name, "Ok");
    assert_eq!(construction.fields.len(), 1);
    assert_eq!(construction.fields[0].0, "value");
}

#[test]
fn m1_3_parse_nullary_construction() {
    let input = "(Red)";
    let form = Parser::parse_str(input).unwrap();
    let ctor_names = vec!["Red".to_string()];
    let result = adt::extract_construction(&form, &ctor_names);
    assert!(result.is_some());
    let construction = result.unwrap().unwrap();
    assert_eq!(construction.ctor_name, "Red");
    assert!(construction.fields.is_empty());
}

// ============================================================
// M1-4: Well-formedness diagnostics
// ============================================================

#[test]
fn m1_4_unknown_constructor() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let cons = adt::Construction {
        ctor_name: "Unknown".to_string(),
        fields: vec![],
        pos: crate::error::Position::new(0, 10, 5),
    };
    let err = adt::check_construction(&cons, &adt_def, "test.lfe").unwrap_err();
    match err {
        crate::error::CheckError::Diagnostic { message, pos, .. } => {
            assert!(message.contains("unknown constructor"), "msg: {message}");
            assert_eq!(pos.line, 10);
            assert_eq!(pos.column, 5);
        }
    }
}

#[test]
fn m1_4_unknown_field() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let cons = adt::Construction {
        ctor_name: "Ok".to_string(),
        fields: vec![(
            "bogus".to_string(),
            SExp::Number(Number::new("1", crate::error::Position::new(0, 1, 1))),
        )],
        pos: crate::error::Position::new(0, 15, 3),
    };
    let err = adt::check_construction(&cons, &adt_def, "test.lfe").unwrap_err();
    match err {
        crate::error::CheckError::Diagnostic { message, pos, .. } => {
            assert!(message.contains("unknown field"), "msg: {message}");
            assert_eq!(pos.line, 15);
            assert_eq!(pos.column, 3);
        }
    }
}

#[test]
fn m1_4_missing_field() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let cons = adt::Construction {
        ctor_name: "Ok".to_string(),
        fields: vec![],
        pos: crate::error::Position::new(0, 20, 1),
    };
    let err = adt::check_construction(&cons, &adt_def, "test.lfe").unwrap_err();
    match err {
        crate::error::CheckError::Diagnostic { message, pos, .. } => {
            assert!(
                message.contains("expects 1 field") || message.contains("missing field"),
                "msg: {message}"
            );
            assert_eq!(pos.line, 20);
        }
    }
}

#[test]
fn m1_4_wrong_arity() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let dp = crate::error::Position::new(0, 1, 1);
    let cons = adt::Construction {
        ctor_name: "Ok".to_string(),
        fields: vec![
            ("value".to_string(), SExp::Number(Number::new("1", dp))),
            ("extra".to_string(), SExp::Number(Number::new("2", dp))),
        ],
        pos: crate::error::Position::new(0, 25, 1),
    };
    let err = adt::check_construction(&cons, &adt_def, "test.lfe").unwrap_err();
    match err {
        crate::error::CheckError::Diagnostic { message, pos, .. } => {
            assert!(message.contains("expects 1 field"), "msg: {message}");
            assert_eq!(pos.line, 25);
        }
    }
}

// ============================================================
// M1-5..M1-9: Lowering backends + snake_case
// ============================================================

#[test]
fn m1_5_snake_case_helper() {
    assert_eq!(lower::to_snake_case("Ok"), "ok");
    assert_eq!(lower::to_snake_case("Error"), "error");
    assert_eq!(lower::to_snake_case("SuperUser"), "super_user");
    assert_eq!(lower::to_snake_case("HTTPServer"), "http_server");
    assert_eq!(lower::to_snake_case("Red"), "red");
    assert_eq!(lower::to_snake_case("CustomerId"), "customer_id");
    assert_eq!(lower::to_snake_case("already_snake"), "already_snake");
    assert_eq!(lower::to_snake_case("A"), "a");
}

#[test]
fn m1_5_lower_tagged_tuple() {
    let adt_input =
        r#"(deftype (result ok err) (repr tagged-tuple) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();
    let ctor_def = adt_def.find_ctor("Ok").unwrap();
    let dp = crate::error::Position::new(0, 1, 1);
    let cons = adt::Construction {
        ctor_name: "Ok".to_string(),
        fields: vec![("value".to_string(), SExp::Number(Number::new("42", dp)))],
        pos: dp,
    };

    let lowered = lower::lower_construction(&cons, ctor_def, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 3);
            assert!(matches!(&l.elements[0], SExp::Symbol(s) if s.value == "tuple"));
            // Tag must be snake_cased: 'ok', not 'Ok'
            match &l.elements[1] {
                SExp::List(q) => {
                    assert!(matches!(&q.elements[1], SExp::Symbol(s) if s.value == "ok"));
                }
                _ => panic!("expected quoted tag"),
            }
        }
        _ => panic!("expected tuple form"),
    }
}

#[test]
fn m1_5_lower_tagged_tuple_multi_word() {
    let adt_input = r#"(deftype role (repr tagged-tuple) (SuperUser (level integer)) (Guest))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();
    let ctor_def = adt_def.find_ctor("SuperUser").unwrap();
    let dp = crate::error::Position::new(0, 1, 1);
    let cons = adt::Construction {
        ctor_name: "SuperUser".to_string(),
        fields: vec![("level".to_string(), SExp::Number(Number::new("5", dp)))],
        pos: dp,
    };

    let lowered = lower::lower_construction(&cons, ctor_def, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 3);
            match &l.elements[1] {
                SExp::List(q) => {
                    assert_eq!(q.elements[1].position().line, 0);
                    assert!(
                        matches!(&q.elements[1], SExp::Symbol(s) if s.value == "super_user"),
                        "expected 'super_user', got: {:?}",
                        q.elements[1]
                    );
                }
                _ => panic!("expected quoted tag"),
            }
        }
        _ => panic!("expected tuple form"),
    }
}

#[test]
fn m1_6_lower_enum() {
    let adt_input = "(deftype colour (repr enum) (Red) (Green) (Blue))";
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();
    let ctor_def = adt_def.find_ctor("Red").unwrap();
    let dp = crate::error::Position::new(0, 1, 1);
    let cons = adt::Construction {
        ctor_name: "Red".to_string(),
        fields: vec![],
        pos: dp,
    };

    let lowered = lower::lower_construction(&cons, ctor_def, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert_eq!(l.elements.len(), 2);
            assert!(matches!(&l.elements[0], SExp::Symbol(s) if s.value == "quote"));
            assert!(matches!(&l.elements[1], SExp::Symbol(s) if s.value == "red"));
        }
        _ => panic!("expected quoted atom, got: {:?}", lowered),
    }
}

#[test]
fn m1_7_lower_transparent() {
    let adt_input = "(deftype customer-id (repr transparent) (CustomerId (v integer)))";
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();
    let ctor_def = adt_def.find_ctor("CustomerId").unwrap();
    let dp = crate::error::Position::new(0, 1, 1);
    let cons = adt::Construction {
        ctor_name: "CustomerId".to_string(),
        fields: vec![("v".to_string(), SExp::Number(Number::new("7", dp)))],
        pos: dp,
    };

    let lowered = lower::lower_construction(&cons, ctor_def, &adt_def, 28);
    match &lowered {
        SExp::Number(n) => assert_eq!(n.value, "7"),
        _ => panic!("expected number, got: {:?}", lowered),
    }
}

#[test]
fn m1_9_default_repr_resolution() {
    let nullary_input = "(deftype colour (Red) (Green) (Blue))";
    let nullary = adt::extract_deftype(&Parser::parse_str(nullary_input).unwrap()).unwrap();
    assert_eq!(nullary.effective_repr(28), adt::ReprKind::Enum);

    let newtype_input = "(deftype customer-id (CustomerId (v integer)))";
    let newtype = adt::extract_deftype(&Parser::parse_str(newtype_input).unwrap()).unwrap();
    assert_eq!(newtype.effective_repr(28), adt::ReprKind::Transparent);

    let sum_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let sum = adt::extract_deftype(&Parser::parse_str(sum_input).unwrap()).unwrap();
    assert_eq!(sum.effective_repr(28), adt::ReprKind::TaggedTuple);
    assert_eq!(sum.effective_repr(29), adt::ReprKind::NativeRecord);
}
