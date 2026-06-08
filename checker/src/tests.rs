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
        other => panic!("unexpected error: {:?}", other),
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
        other => panic!("unexpected error: {:?}", other),
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
        other => panic!("unexpected error: {:?}", other),
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
        other => panic!("unexpected error: {:?}", other),
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
        other => panic!("unexpected error: {:?}", other),
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

// ============================================================
// M2 tests — Pattern matching & exhaustiveness
// ============================================================

#[test]
fn m2_1_parse_case_typed() {
    let input = r#"(case/typed x
  ((Ok v) v)
  ((Error e) e))"#;
    let form = Parser::parse_str(input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();
    assert_eq!(tm.clauses.len(), 2);
    match &tm.clauses[0].pattern {
        crate::matching::Pattern::Constructor { name, bindings, .. } => {
            assert_eq!(name, "Ok");
            assert_eq!(bindings, &["v"]);
        }
        _ => panic!("expected constructor pattern"),
    }
}

#[test]
fn m2_1_parse_wildcard_and_var() {
    let input = r#"(case/typed x
  ((Ok v) v)
  (_ 0))"#;
    let form = Parser::parse_str(input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();
    assert!(tm.clauses[1].pattern.is_catch_all());
}

#[test]
fn m2_1_parse_explicit_type_annotation() {
    let input = r#"(case/typed x :type result
  ((Ok v) v)
  ((Error e) e))"#;
    let form = Parser::parse_str(input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();
    assert_eq!(tm.scrutinee_type, Some("result".to_string()));
}

#[test]
fn m2_3_exhaustiveness_rejects_missing() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok v) v))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_exhaustiveness(&tm, &adt_def, "test.lfe");
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        crate::error::CheckError::NonExhaustive {
            missing, type_name, ..
        } => {
            assert_eq!(type_name, "result");
            assert_eq!(missing, &["Error"]);
        }
        _ => panic!("expected NonExhaustive"),
    }
}

#[test]
fn m2_3_exhaustiveness_accepts_complete() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok v) v)
  ((Error e) e))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_exhaustiveness(&tm, &adt_def, "test.lfe");
    assert!(errors.is_empty());
}

#[test]
fn m2_3_exhaustiveness_accepts_wildcard() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok v) v)
  (_ 0))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_exhaustiveness(&tm, &adt_def, "test.lfe");
    assert!(errors.is_empty());
}

#[test]
fn m2_3_exhaustiveness_missing_multiple() {
    let adt_input = "(deftype status (Pending) (Shipped) (Delivered) (Cancelled))";
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed s
  ((Pending) 1))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_exhaustiveness(&tm, &adt_def, "test.lfe");
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        crate::error::CheckError::NonExhaustive { missing, .. } => {
            assert_eq!(missing, &["Shipped", "Delivered", "Cancelled"]);
        }
        _ => panic!("expected NonExhaustive"),
    }
}

#[test]
fn m2_4_pattern_unknown_ctor() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok v) v)
  ((Unknown e) e))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_pattern_wellformedness(&tm, &adt_def, "test.lfe");
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(
                message.contains("unknown constructor `Unknown`"),
                "msg: {message}"
            );
        }
        _ => panic!("expected Diagnostic"),
    }
}

#[test]
fn m2_4_pattern_wrong_arity() {
    let adt_input = r#"(deftype (result ok err) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok a b) a)
  ((Error e) e))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let errors = crate::matching::check_pattern_wellformedness(&tm, &adt_def, "test.lfe");
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(message.contains("has 1 field"), "msg: {message}");
        }
        _ => panic!("expected Diagnostic"),
    }
}

#[test]
fn m2_8_lower_case_tagged_tuple() {
    let adt_input =
        r#"(deftype (result ok err) (repr tagged-tuple) (Ok (value ok)) (Error (reason err)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed x
  ((Ok v) v)
  ((Error e) e))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let lowered = crate::match_lower::lower_case_typed(&tm, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert!(matches!(&l.elements[0], SExp::Symbol(s) if s.value == "case"));
            assert!(l.elements.len() >= 3);
        }
        _ => panic!("expected case list"),
    }
}

#[test]
fn m2_10_redundancy_warning() {
    let match_input = r#"(case/typed x
  ((Ok v) v)
  ((Ok w) w)
  ((Error e) e))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let warnings = crate::matching::check_redundancy(&tm, "test.lfe");
    assert_eq!(warnings.len(), 1);
    match &warnings[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(message.contains("redundant"), "msg: {message}");
        }
        _ => panic!("expected Diagnostic"),
    }
}

#[test]
fn m2_10_unreachable_after_wildcard() {
    let match_input = r#"(case/typed x
  (_ 0)
  ((Ok v) v))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let warnings = crate::matching::check_redundancy(&tm, "test.lfe");
    assert_eq!(warnings.len(), 1);
    match &warnings[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(message.contains("unreachable"), "msg: {message}");
        }
        _ => panic!("expected Diagnostic"),
    }
}

#[test]
fn m2_6_diagnostic_engine_renders() {
    let err = crate::error::CheckError::NonExhaustive {
        file: "test.lfe".to_string(),
        pos: crate::error::Position::new(0, 5, 3),
        type_name: "result".to_string(),
        missing: vec!["Error".to_string()],
    };
    let mut collector = crate::diagnostic::DiagnosticCollector::new();
    collector.add_check_error(&err, Some("line1\nline2\nline3\nline4\n(case/typed r"));
    assert!(collector.has_errors());

    let human = collector.render_human();
    assert!(human.contains("non-exhaustive"), "human: {human}");
    assert!(human.contains("Error"), "human missing ctor: {human}");
    assert!(human.contains("Hint:"), "human missing hint: {human}");

    let json = collector.render_json();
    assert!(json.contains("\"code\":\"E100\""), "json: {json}");
    assert!(json.contains("\"Error\""), "json missing ctor: {json}");
}

#[test]
fn m2_11_snapshot_non_exhaustive_human() {
    let err = crate::error::CheckError::NonExhaustive {
        file: "orders.tlfe".to_string(),
        pos: crate::error::Position::new(0, 10, 1),
        type_name: "status".to_string(),
        missing: vec!["Shipped".to_string(), "Cancelled".to_string()],
    };
    let mut collector = crate::diagnostic::DiagnosticCollector::new();
    collector.add_check_error(&err, Some(&("x\n".repeat(9) + "(case/typed s")));
    let human = collector.render_human();
    let expected = concat!(
        "error[E100]: non-exhaustive pattern match on type `status`\n",
        "  --> orders.tlfe:10:1\n",
        "      |\n",
        "   10 | (case/typed s\n",
        "      | ^\n",
        "   |\n",
        "   = These values are not matched:\n",
        "       - Shipped\n",
        "       - Cancelled\n",
        "   = Hint: add clauses for the missing constructor(s), or use `_` as a catch-all.\n",
        "\n",
    );
    assert_eq!(
        human, expected,
        "\n--- GOT ---\n{human}\n--- EXPECTED ---\n{expected}"
    );
}

#[test]
fn m2_11_snapshot_non_exhaustive_json() {
    let err = crate::error::CheckError::NonExhaustive {
        file: "orders.tlfe".to_string(),
        pos: crate::error::Position::new(0, 10, 1),
        type_name: "status".to_string(),
        missing: vec!["Shipped".to_string(), "Cancelled".to_string()],
    };
    let mut collector = crate::diagnostic::DiagnosticCollector::new();
    collector.add_check_error(&err, None);
    let json = collector.render_json();
    assert!(json.contains("\"code\":\"E100\""));
    assert!(json.contains("\"severity\":\"error\""));
    assert!(json.contains("\"file\":\"orders.tlfe\""));
    assert!(json.contains("\"line\":10"));
    assert!(json.contains("\"missing_ctors\":[\"Shipped\",\"Cancelled\"]"));
    assert!(json.contains("\"hint\":\"add clauses"));
}

// ============================================================
// M2-9: enum + transparent match lowering tests
// ============================================================

#[test]
fn m2_9_lower_case_enum() {
    let adt_input = "(deftype colour (repr enum) (Red) (Green) (Blue))";
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed c
  ((Red) 1)
  ((Green) 2)
  ((Blue) 3))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let lowered = crate::match_lower::lower_case_typed(&tm, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert!(matches!(&l.elements[0], SExp::Symbol(s) if s.value == "case"));
            // First clause pattern should be a quoted snake_cased atom 'red'
            match &l.elements[2] {
                SExp::List(clause) => match &clause.elements[0] {
                    SExp::List(q) => {
                        assert!(matches!(&q.elements[0], SExp::Symbol(s) if s.value == "quote"));
                        assert!(
                            matches!(&q.elements[1], SExp::Symbol(s) if s.value == "red"),
                            "expected 'red', got: {:?}",
                            q.elements[1]
                        );
                    }
                    _ => panic!("expected quoted atom pattern"),
                },
                _ => panic!("expected clause list"),
            }
        }
        _ => panic!("expected case list"),
    }
}

#[test]
fn m2_9_lower_case_transparent() {
    let adt_input = "(deftype customer-id (repr transparent) (CustomerId (v integer)))";
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();

    let match_input = r#"(case/typed id
  ((CustomerId v) v))"#;
    let form = Parser::parse_str(match_input).unwrap();
    let tm = crate::matching::extract_case_typed(&form).unwrap();

    let lowered = crate::match_lower::lower_case_typed(&tm, &adt_def, 28);
    match &lowered {
        SExp::List(l) => {
            assert!(matches!(&l.elements[0], SExp::Symbol(s) if s.value == "case"));
            // Transparent pattern: just a variable binding (no tuple/tag)
            match &l.elements[2] {
                SExp::List(clause) => {
                    assert!(
                        matches!(&clause.elements[0], SExp::Symbol(s) if s.value == "v"),
                        "expected bare variable 'v', got: {:?}",
                        clause.elements[0]
                    );
                }
                _ => panic!("expected clause list"),
            }
        }
        _ => panic!("expected case list"),
    }
}

// ============================================================
// M3 tests — Contracts & Bidirectional Body Checking
// ============================================================

#[test]
fn m3_1_synth_literals() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let (ty, errs) = crate::typecheck::synth_expr(
        &SExp::Number(Number::new("42", crate::error::Position::new(0, 1, 1))),
        &env,
        &tenv,
        "test.lfe",
    );
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::Integer);

    let (ty, _) = crate::typecheck::synth_expr(
        &SExp::String(StringLit::new(
            "hello",
            crate::error::Position::new(0, 1, 1),
        )),
        &env,
        &tenv,
        "test.lfe",
    );
    assert_eq!(ty, crate::typecheck::Type::String);
}

#[test]
fn m3_1_synth_var_from_env() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.bind_var("x", crate::typecheck::Type::Integer);
    let tenv = TypeEnv::new();

    let (ty, errs) = crate::typecheck::synth_expr(
        &SExp::Symbol(Symbol::new("x", crate::error::Position::new(0, 1, 1))),
        &env,
        &tenv,
        "test.lfe",
    );
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::Integer);
}

#[test]
fn m3_2_synth_typed_call() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.register_fun(
        "greet",
        crate::typecheck::FunSig {
            args: vec![("name".to_string(), crate::typecheck::Type::String)],
            returns: crate::typecheck::Type::String,
        },
    );
    let tenv = TypeEnv::new();

    let input = r#"(greet "world")"#;
    let form = Parser::parse_str(input).unwrap();
    let (ty, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::String);
}

#[test]
fn m3_3_body_return_mismatch() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let body = vec![SExp::Number(Number::new(
        "42",
        crate::error::Position::new(0, 5, 3),
    ))];
    let errors = crate::typecheck::check_body_return(
        &body,
        "binary",
        &env,
        &tenv,
        "test.lfe",
        crate::error::Position::new(0, 1, 1),
    );
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(message.contains("integer"), "msg: {message}");
            assert!(message.contains("binary"), "msg: {message}");
        }
        other => panic!("expected Diagnostic, got: {:?}", other),
    }
}

#[test]
fn m3_3_body_return_match() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let body = vec![SExp::Number(Number::new(
        "42",
        crate::error::Position::new(0, 1, 1),
    ))];
    let errors = crate::typecheck::check_body_return(
        &body,
        "integer",
        &env,
        &tenv,
        "test.lfe",
        crate::error::Position::new(0, 1, 1),
    );
    assert!(errors.is_empty());
}

#[test]
fn m3_4_call_arg_type_mismatch() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.register_fun(
        "add",
        crate::typecheck::FunSig {
            args: vec![
                ("a".to_string(), crate::typecheck::Type::Integer),
                ("b".to_string(), crate::typecheck::Type::Integer),
            ],
            returns: crate::typecheck::Type::Integer,
        },
    );
    let tenv = TypeEnv::new();

    let input = r#"(add "hello" 2)"#;
    let form = Parser::parse_str(input).unwrap();
    let (_, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert_eq!(errs.len(), 1);
    match &errs[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(
                message.contains("expected type `integer`"),
                "msg: {message}"
            );
            assert!(message.contains("got `string`"), "msg: {message}");
        }
        other => panic!("expected Diagnostic, got: {:?}", other),
    }
}

#[test]
fn m3_4_call_wrong_arity() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.register_fun(
        "add",
        crate::typecheck::FunSig {
            args: vec![
                ("a".to_string(), crate::typecheck::Type::Integer),
                ("b".to_string(), crate::typecheck::Type::Integer),
            ],
            returns: crate::typecheck::Type::Integer,
        },
    );
    let tenv = TypeEnv::new();

    let input = "(add 1)";
    let form = Parser::parse_str(input).unwrap();
    let (_, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert_eq!(errs.len(), 1);
    match &errs[0] {
        crate::error::CheckError::Diagnostic { message, .. } => {
            assert!(
                message.contains("expects 2 argument(s), got 1"),
                "msg: {message}"
            );
        }
        other => panic!("expected Diagnostic, got: {:?}", other),
    }
}

#[test]
fn m3_8_prelude_arithmetic() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let input = "(+ 1 2)";
    let form = Parser::parse_str(input).unwrap();
    let (ty, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::Number);
}

#[test]
fn m3_8_prelude_comparison() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let input = "(> 1 2)";
    let form = Parser::parse_str(input).unwrap();
    let (ty, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::Boolean);
}

#[test]
fn m3_9_dynamic_unknown_call() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let input = "(unknown-fn 1 2)";
    let form = Parser::parse_str(input).unwrap();
    let (ty, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "test.lfe");
    assert!(errs.is_empty());
    assert_eq!(ty, crate::typecheck::Type::Dynamic);
}

#[test]
fn m3_9_dynamic_compatible_with_any() {
    assert!(crate::typecheck::types_compatible(
        &crate::typecheck::Type::Dynamic,
        &crate::typecheck::Type::Integer,
    ));
    assert!(crate::typecheck::types_compatible(
        &crate::typecheck::Type::Integer,
        &crate::typecheck::Type::Dynamic,
    ));
}

// ============================================================
// M3-10: Exact golden snapshots for headline diagnostics
// ============================================================

#[test]
fn m3_10_snapshot_return_mismatch() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let body = vec![SExp::Number(Number::new(
        "42",
        crate::error::Position::new(0, 5, 8),
    ))];
    let errors = crate::typecheck::check_body_return(
        &body,
        "binary",
        &env,
        &tenv,
        "greeting.tlfe",
        crate::error::Position::new(0, 3, 1),
    );
    assert_eq!(errors.len(), 1);
    let msg = format!("{}", errors[0]);
    assert_eq!(
        msg,
        "greeting.tlfe:3:1: body returns `integer`, but contract declares `:returns binary`"
    );
}

#[test]
fn m3_10_snapshot_arg_type_mismatch() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.register_fun(
        "add",
        crate::typecheck::FunSig {
            args: vec![
                ("a".to_string(), crate::typecheck::Type::Integer),
                ("b".to_string(), crate::typecheck::Type::Integer),
            ],
            returns: crate::typecheck::Type::Integer,
        },
    );
    let tenv = TypeEnv::new();

    let input = r#"(add "hello" 2)"#;
    let form = Parser::parse_str(input).unwrap();
    let (_, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "math.tlfe");
    assert_eq!(errs.len(), 1);
    let msg = format!("{}", errs[0]);
    assert_eq!(
        msg,
        "math.tlfe:1:6: argument `a` expected type `integer`, got `string`"
    );
}

#[test]
fn m3_10_snapshot_wrong_arity() {
    let mut env = crate::typecheck::BodyEnv::new();
    env.register_fun(
        "add",
        crate::typecheck::FunSig {
            args: vec![
                ("a".to_string(), crate::typecheck::Type::Integer),
                ("b".to_string(), crate::typecheck::Type::Integer),
            ],
            returns: crate::typecheck::Type::Integer,
        },
    );
    let tenv = TypeEnv::new();

    let input = "(add 1)";
    let form = Parser::parse_str(input).unwrap();
    let (_, errs) = crate::typecheck::synth_expr(&form, &env, &tenv, "math.tlfe");
    assert_eq!(errs.len(), 1);
    let msg = format!("{}", errs[0]);
    assert_eq!(msg, "math.tlfe:1:1: function expects 2 argument(s), got 1");
}

#[test]
fn m3_10_snapshot_field_value_mismatch() {
    let adt_input = r#"(deftype (box t) (repr tagged-tuple) (Box (val integer)))"#;
    let adt_def = adt::extract_deftype(&Parser::parse_str(adt_input).unwrap()).unwrap();
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let dp = crate::error::Position::new(0, 10, 3);
    let errs = crate::typecheck::check_constructor_field_values(
        "Box",
        &[("val".to_string(), SExp::String(StringLit::new("oops", dp)))],
        &adt_def,
        &env,
        &tenv,
        "container.tlfe",
        dp,
    );
    assert_eq!(errs.len(), 1);
    let msg = format!("{}", errs[0]);
    assert_eq!(
        msg,
        "container.tlfe:10:3: field `val` of constructor `Box` expects type `integer`, got `string`"
    );
}

// ============================================================
// M3.5 tests — Cleanup (C-1..C-5)
// ============================================================

#[test]
fn c2_branch_body_typing_rejects_wrong_type() {
    let bodies = vec![
        SExp::String(StringLit::new(
            "hello",
            crate::error::Position::new(0, 5, 3),
        )),
        SExp::Number(Number::new("42", crate::error::Position::new(0, 6, 3))),
    ];
    let expected = crate::typecheck::Type::Atom;
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let errs =
        crate::typecheck::check_case_typed_branches(&bodies, &expected, &env, &tenv, "test.tlfe");
    assert_eq!(errs.len(), 2);
    let msg0 = format!("{}", errs[0]);
    assert_eq!(
        msg0,
        "test.tlfe:5:3: case/typed branch returns `string`, but expected `atom`"
    );
    let msg1 = format!("{}", errs[1]);
    assert_eq!(
        msg1,
        "test.tlfe:6:3: case/typed branch returns `integer`, but expected `atom`"
    );
}

#[test]
fn c2_branch_body_typing_accepts_matching() {
    let bodies = vec![SExp::List(crate::sexp::types::List::new(
        vec![
            SExp::Symbol(Symbol::new("quote", crate::error::Position::new(0, 1, 1))),
            SExp::Symbol(Symbol::new("ok", crate::error::Position::new(0, 1, 2))),
        ],
        crate::error::Position::new(0, 1, 1),
    ))];
    let expected = crate::typecheck::Type::Atom;
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let errs =
        crate::typecheck::check_case_typed_branches(&bodies, &expected, &env, &tenv, "test.tlfe");
    assert!(errs.is_empty());
}

#[test]
fn c5_poly_identity_accepted() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let args = vec![("x".to_string(), "a".to_string())];
    let arg_values = vec![SExp::Number(Number::new(
        "42",
        crate::error::Position::new(0, 1, 5),
    ))];
    let errs = crate::typecheck::check_poly_contract(
        &args,
        &arg_values,
        "a",
        &env,
        &tenv,
        "test.tlfe",
        crate::error::Position::new(0, 1, 1),
    );
    assert!(errs.is_empty());
}

#[test]
fn c5_poly_same_var_different_types_rejected() {
    let env = crate::typecheck::BodyEnv::new();
    let tenv = TypeEnv::new();

    let args = vec![
        ("x".to_string(), "a".to_string()),
        ("y".to_string(), "a".to_string()),
    ];
    let dp = crate::error::Position::new(0, 1, 1);
    let arg_values = vec![
        SExp::Number(Number::new("42", dp)),
        SExp::String(StringLit::new(
            "hello",
            crate::error::Position::new(0, 1, 10),
        )),
    ];
    let errs = crate::typecheck::check_poly_contract(
        &args,
        &arg_values,
        "a",
        &env,
        &tenv,
        "poly.tlfe",
        dp,
    );
    assert_eq!(errs.len(), 1);
    let msg = format!("{}", errs[0]);
    assert_eq!(
        msg,
        "poly.tlfe:1:10: type variable `a` bound to `integer` by argument `x`, but got `string` here"
    );
}

// ============================================================
// C-1: Exact render snapshot through DiagnosticCollector
// ============================================================

#[test]
fn c1_snapshot_return_mismatch_rendered_human() {
    let err = crate::error::CheckError::Diagnostic {
        file: "greeting.tlfe".to_string(),
        pos: crate::error::Position::new(0, 3, 1),
        message: "body returns `integer`, but contract declares `:returns binary`".to_string(),
    };
    let source = "line1\nline2\n(defun/typed oops";
    let mut collector = crate::diagnostic::DiagnosticCollector::new();
    collector.add_check_error(&err, Some(source));
    let human = collector.render_human();
    let expected = concat!(
        "error[E001]: body returns `integer`, but contract declares `:returns binary`\n",
        "  --> greeting.tlfe:3:1\n",
        "     |\n",
        "   3 | (defun/typed oops\n",
        "     | ^\n",
        "\n",
    );
    assert_eq!(human, expected);
}

#[test]
fn c1_snapshot_return_mismatch_rendered_json() {
    let err = crate::error::CheckError::Diagnostic {
        file: "greeting.tlfe".to_string(),
        pos: crate::error::Position::new(0, 3, 1),
        message: "body returns `integer`, but contract declares `:returns binary`".to_string(),
    };
    let mut collector = crate::diagnostic::DiagnosticCollector::new();
    collector.add_check_error(&err, None);
    let json = collector.render_json();
    assert_eq!(
        json,
        concat!(
            "[{",
            "\"code\":\"E001\",",
            "\"severity\":\"error\",",
            "\"file\":\"greeting.tlfe\",",
            "\"line\":3,",
            "\"column\":1,",
            "\"message\":\"body returns `integer`, but contract declares `:returns binary`\",",
            "\"missing_ctors\":[],",
            "\"hint\":null",
            "}]"
        )
    );
}

// ============================================================
// M3.5 iteration 3: Binary ≠ String soundness test
// ============================================================

#[test]
fn c4_binary_not_compatible_with_string() {
    assert!(!crate::typecheck::types_compatible(
        &crate::typecheck::Type::Binary,
        &crate::typecheck::Type::String,
    ));
    assert!(!crate::typecheck::types_compatible(
        &crate::typecheck::Type::String,
        &crate::typecheck::Type::Binary,
    ));
}

#[test]
fn c4_string_still_compatible_with_list() {
    assert!(crate::typecheck::types_compatible(
        &crate::typecheck::Type::String,
        &crate::typecheck::Type::List,
    ));
    assert!(crate::typecheck::types_compatible(
        &crate::typecheck::Type::List,
        &crate::typecheck::Type::String,
    ));
}
