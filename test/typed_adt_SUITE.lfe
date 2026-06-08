;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0
;;
;; Unless required by applicable law or agreed to in writing, software
;; distributed under the License is distributed on an "AS IS" BASIS,
;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;; See the License for the specific language governing permissions and
;; limitations under the License.

;; File    : typed_adt_SUITE.lfe
;; Purpose : End-to-end tests for ADT parsing, lowering, repr backends,
;;           registry emission, and line-injection regression (M1).

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_adt_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; M1-5: tagged-tuple
   (m1_5_tagged_tuple 1)
   (m1_5_tagged_tuple_multi_word 1)
   ;; M1-6: enum
   (m1_6_enum 1)
   ;; M1-7: transparent
   (m1_7_transparent 1)
   ;; M1-10: registry emission
   (m1_10_registry_attr 1)
   ;; M1-11: backend matrix (exact representation assertions)
   (m1_11_matrix_tagged_tuple 1)
   (m1_11_matrix_enum 1)
   (m1_11_matrix_transparent 1)
   ;; M1-12: line injection regression
   (m1_12_adt_crash_line_injection 1)
   (m1_12_adt_error_line_injection 1)))

(defun all ()
  '(m1_5_tagged_tuple
    m1_5_tagged_tuple_multi_word
    m1_6_enum
    m1_7_transparent
    m1_10_registry_attr
    m1_11_matrix_tagged_tuple
    m1_11_matrix_enum
    m1_11_matrix_transparent
    m1_12_adt_crash_line_injection
    m1_12_adt_error_line_injection))

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

;;; ============================================================
;;; M1-5: tagged-tuple backend
;;; ============================================================

(defun m1_5_tagged_tuple (config)
  (let* ((forms (check-and-decode config "adt/tagged_tuple" "shapes.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "shapes.lfet" priv-dir)
      (`#(ok shapes ,beam-bin)
       (code:purge 'shapes)
       (let ((`#(module shapes) (code:load_binary 'shapes "shapes.beam" beam-bin)))
         (let ((result (call 'shapes 'make-ok 42)))
           ;; snake_cased tag: ok, not Ok
           (case result
             (#(ok 42) 'ok)
             (other (ct:fail `#(wrong_tagged_tuple ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

(defun m1_5_tagged_tuple_multi_word (config)
  (let* ((forms (check-and-decode config "adt/tagged_tuple" "roles.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "roles.lfet" priv-dir)
      (`#(ok roles ,beam-bin)
       (code:purge 'roles)
       (let ((`#(module roles) (code:load_binary 'roles "roles.beam" beam-bin)))
         ;; SuperUser → super_user (true snake_case, not superuser)
         (let ((sup (call 'roles 'make-super 5))
               (reg (call 'roles 'make-regular)))
           (case sup
             (#(super_user 5)
              (case reg
                ('regular_user 'ok)
                (other (ct:fail `#(wrong_regular ,other)))))
             (other (ct:fail `#(wrong_super_user ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; M1-6: enum backend
;;; ============================================================

(defun m1_6_enum (config)
  (let* ((forms (check-and-decode config "adt/enum" "colours.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "colours.lfet" priv-dir)
      (`#(ok colours ,beam-bin)
       (code:purge 'colours)
       (let ((`#(module colours) (code:load_binary 'colours "colours.beam" beam-bin)))
         (let ((result (call 'colours 'get-red)))
           (case result
             ('red 'ok)
             (other (ct:fail `#(wrong_enum_value ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; M1-7: transparent backend
;;; ============================================================

(defun m1_7_transparent (config)
  (let* ((forms (check-and-decode config "adt/transparent" "ids.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "ids.lfet" priv-dir)
      (`#(ok ids ,beam-bin)
       (code:purge 'ids)
       (let ((`#(module ids) (code:load_binary 'ids "ids.beam" beam-bin)))
         (let ((result (call 'ids 'make-id 7)))
           (case (=:= result 7)
             ('true 'ok)
             ('false (ct:fail `#(wrong_transparent_value ,result)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; M1-10: registry emission
;;; ============================================================

(defun m1_10_registry_attr (config)
  (let* ((forms (check-and-decode config "adt/tagged_tuple" "shapes.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "shapes.lfet" priv-dir)
      (`#(ok shapes ,beam-bin)
       (let ((`#(ok #(shapes ,chunks))
              (beam_lib:chunks beam-bin '(attributes))))
         (let* ((attrs (proplists:get_value 'attributes chunks))
                (registry (proplists:get_value 'typed-registry attrs)))
           (case registry
             ('undefined (ct:fail '#(no_registry_attr)))
             (_ 'ok)))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; M1-11: backend matrix
;;; ============================================================

(defun m1_11_matrix_tagged_tuple (config)
  (let* ((forms (check-and-decode config "adt/tagged_tuple" "shapes.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "shapes.lfet" priv-dir)
      (`#(ok shapes ,beam-bin)
       (code:purge 'shapes)
       (code:load_binary 'shapes "shapes.beam" beam-bin)
       (let ((result (call 'shapes 'make-ok 99)))
         ;; Exact representation: {ok, 99} — snake_cased tag, flat tuple
         (case result
           (#(ok 99) 'ok)
           (other (ct:fail `#(wrong_exact_repr ,other))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

(defun m1_11_matrix_enum (config)
  (let* ((forms (check-and-decode config "adt/enum" "colours.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "colours.lfet" priv-dir)
      (`#(ok colours ,beam-bin)
       (code:purge 'colours)
       (code:load_binary 'colours "colours.beam" beam-bin)
       (let ((result (call 'colours 'get-red)))
         ;; Exact representation: the atom 'red' (snake_cased)
         (case result
           ('red 'ok)
           (other (ct:fail `#(wrong_exact_repr ,other))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

(defun m1_11_matrix_transparent (config)
  (let* ((forms (check-and-decode config "adt/transparent" "ids.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "ids.lfet" priv-dir)
      (`#(ok ids ,beam-bin)
       (code:purge 'ids)
       (code:load_binary 'ids "ids.beam" beam-bin)
       (let ((result (call 'ids 'make-id 42)))
         ;; Exact representation: the integer 42 (wrapper erased)
         (case result
           (42 'ok)
           (other (ct:fail `#(wrong_exact_repr ,other))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; M1-12: line injection regression
;;; ============================================================

(defun m1_12_adt_crash_line_injection (config)
  (let* ((forms (check-and-decode config "adt/crash" "adt_boom.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "adt_boom.lfet" priv-dir)
      (`#(ok adt_boom ,beam-bin)
       (code:purge 'adt_boom)
       (let ((`#(module adt_boom) (code:load_binary 'adt_boom "adt_boom.beam" beam-bin)))
         (try (adt_boom:explode)
           (catch
             (`#(error adt-bang ,stacktrace)
              (let ((`(#(adt_boom explode 0 ,info) . ,_) stacktrace))
                (let ((file (proplists:get_value 'file info))
                      (line (proplists:get_value 'line info)))
                  (case (tuple file line)
                    (#("adt_boom.lfet" 20) 'ok)
                    (other (ct:fail `#(wrong_file_line ,other)))))))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

(defun m1_12_adt_error_line_injection (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir (proplists:get_value 'priv_dir config))
         (fixture (filename:join (list fixture-dir "adt" "malformed" "bad_ctor.lfet")))
         (eetf-file (filename:join priv-dir "bad_ctor.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       (case (string:find output "18:")
         ('nomatch (ct:fail `#(diagnostic_missing_line ,output)))
         (_ 'ok)))
      ('false
       (ct:fail '#(checker_should_have_failed))))))

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
