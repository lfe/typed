;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0

;; File    : typed_matching_SUITE.lfe
;; Purpose : End-to-end tests for case/typed, exhaustiveness, field access,
;;           match lowering, and line-injection regression (M2).

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_matching_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; M2-3/M2-8: exhaustive match compiles + runs
   (m2_3_exhaustive_match 1)
   ;; M2-3: non-exhaustive rejected
   (m2_3_non_exhaustive_rejected 1)
   ;; M2-5: field access via patterns
   (m2_5_field_access 1)
   ;; M2-9: backend matrix for matching
   (m2_9_matrix_enum_match 1)
   (m2_9_matrix_transparent_match 1)
   ;; M2-12: line injection regression
   (m2_12_match_line_injection 1)))

(defun all ()
  '(m2_3_exhaustive_match
    m2_3_non_exhaustive_rejected
    m2_5_field_access
    m2_9_matrix_enum_match
    m2_9_matrix_transparent_match
    m2_12_match_line_injection))

(defun suite () `(#(timetrap #(seconds 60))))

(defun groups () ())

(defun init_per_suite (config)
  (let* ((project-root (find-project-root))
         (checker-bin (filename:join
                       (list project-root "checker" "target" "debug" "typed-check"))))
    (case (filelib:is_file checker-bin)
      ('true
       (let ((fixture-dir (filename:join project-root "test/fixtures")))
         (lists:append
          (list (tuple 'checker_bin checker-bin)
                (tuple 'fixture_dir fixture-dir)
                (tuple 'project_root project-root))
          config)))
      ('false
       (tuple 'skip "typed-check binary not found; run 'cargo build' in checker/")))))

(defun end_per_suite (_config) 'ok)

;;; M2-3/M2-8: exhaustive match compiles and runs correctly

(defun m2_3_exhaustive_match (config)
  (let* ((forms (check-and-decode config "matching/exhaustive" "good_match.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "good_match.tlfe" priv-dir)
      (`#(ok good_match ,beam-bin)
       (code:purge 'good_match)
       (let ((`#(module good_match) (code:load_binary 'good_match "good_match.beam" beam-bin)))
         ;; Test Ok path
         (let ((ok-result (call 'good_match 'unwrap (tuple 'ok 42))))
           (case ok-result
             (42 'ok)
             (other (ct:fail `#(wrong_ok_result ,other)))))
         ;; Test Error path
         (let ((err-result (call 'good_match 'unwrap (tuple 'error "oops"))))
           (case err-result
             (-1 'ok)
             (other (ct:fail `#(wrong_error_result ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M2-3: non-exhaustive match is rejected by the checker

(defun m2_3_non_exhaustive_rejected (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "matching" "non_exhaustive" "missing_ctors.tlfe")))
         (eetf-file   (filename:join priv-dir "missing_ctors.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       ;; Must mention the missing constructors
       (let ((has-error (=/= 'nomatch (string:find output "Error")))
             (has-timeout (=/= 'nomatch (string:find output "Timeout"))))
         (case (andalso has-error has-timeout)
           ('true 'ok)
           ('false (ct:fail `#(missing_ctor_names ,output))))))
      ('false
       (ct:fail '#(checker_should_have_rejected))))))

;;; M2-5: field access via patterns

(defun m2_5_field_access (config)
  (let* ((forms (check-and-decode config "matching/field_access" "extract.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "extract.tlfe" priv-dir)
      (`#(ok extract ,beam-bin)
       (code:purge 'extract)
       (let ((`#(module extract) (code:load_binary 'extract "extract.beam" beam-bin)))
         (let ((result (call 'extract 'get-value (tuple 'wrap 99))))
           (case result
             (99 'ok)
             (other (ct:fail `#(wrong_field_access ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M2-9: backend matrix — enum matching

(defun m2_9_matrix_enum_match (config)
  (let* ((forms (check-and-decode config "matching/matrix" "enum_match.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "enum_match.tlfe" priv-dir)
      (`#(ok enum_match ,beam-bin)
       (code:purge 'enum_match)
       (let ((`#(module enum_match) (code:load_binary 'enum_match "enum_match.beam" beam-bin)))
         ;; Exact assertions: matching on atom values
         (let ((r (call 'enum_match 'colour-code 'red))
               (g (call 'enum_match 'colour-code 'green))
               (b (call 'enum_match 'colour-code 'blue)))
           (case (tuple r g b)
             (#(1 2 3) 'ok)
             (other (ct:fail `#(wrong_enum_match ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M2-9: backend matrix — transparent matching

(defun m2_9_matrix_transparent_match (config)
  (let* ((forms (check-and-decode config "matching/matrix" "transparent_match.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "transparent_match.tlfe" priv-dir)
      (`#(ok transparent_match ,beam-bin)
       (code:purge 'transparent_match)
       (let ((`#(module transparent_match)
              (code:load_binary 'transparent_match "transparent_match.beam" beam-bin)))
         ;; Exact assertion: transparent unwraps the bare value
         (let ((result (call 'transparent_match 'unwrap-id 42)))
           (case result
             (42 'ok)
             (other (ct:fail `#(wrong_transparent_match ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M2-12: line injection regression through case/typed

(defun m2_12_match_line_injection (config)
  (let* ((forms (check-and-decode config "matching/exhaustive" "good_match.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "good_match.tlfe" priv-dir)
      (`#(ok good_match ,beam-bin)
       ;; Check that the function has the right source line in debug_info
       (let ((`#(ok #(good_match ,chunks))
              (beam_lib:chunks beam-bin '(abstract_code))))
         (let* ((ac (proplists:get_value 'abstract_code chunks)))
           (case ac
             (`#(raw_abstract_v1 ,forms-list)
              ;; Find the unwrap function and check its line
              (let ((unwrap-fns (lists:filter
                                 (lambda (f)
                                   (case f
                                     (`#(function ,_ unwrap ,_ ,_) 'true)
                                     (_ 'false)))
                                 forms-list)))
                (case unwrap-fns
                  (`(#(function ,line unwrap ,_ ,_) . ,_)
                   ;; Line should be the defun/typed line from the fixture
                   (case (> line 0)
                     ('true 'ok)
                     ('false (ct:fail `#(bad_line ,line)))))
                  (_ (ct:fail '#(no_unwrap_function))))))
             (_ (ct:fail `#(no_abstract_code ,ac)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; --- helpers ---

(defun check-and-decode (config sub-dir file-name)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir sub-dir file-name)))
         (eetf-file   (filename:join priv-dir
                       (++ (re:replace sub-dir "/" "_" '(global #(return list)))
                           "_" file-name ".eetf")))
         (`#(0 ,_)    (run-checker checker-bin fixture eetf-file))
         (`#(ok ,bin)  (file:read_file eetf-file)))
    (binary_to_term bin)))

(defun run-checker (checker-bin input-file output-file)
  (let* ((cmd (lists:flatten
               (io_lib:format "\"~s\" \"~s\" --output \"~s\" 2>&1; echo \"EXIT:$?\""
                              (list checker-bin input-file output-file))))
         (raw-output (os:cmd cmd))
         (lines (string:split (string:trim raw-output) "\n" 'all))
         (last-line (lists:last lines)))
    (case (string:prefix last-line "EXIT:")
      ('nomatch (tuple 1 raw-output))
      (exit-str
       (let ((code (list_to_integer (string:trim exit-str)))
             (diag-lines (lists:droplast lines)))
         (tuple code (lists:join "\n" diag-lines)))))))

(defun find-project-root ()
  (let ((beam (code:which 'typed_driver)))
    (case beam
      ('non_existing (error 'typed_driver_not_found))
      (path
       (let* ((abs (filename:absname (filename:dirname path)))
              (parts (filename:split abs))
              (`#(,before ,_) (lists:splitwith
                               (lambda (p) (/= p "_build"))
                               parts)))
         (case before
           ('() (filename:dirname abs))
           (_ (filename:join before))))))))
