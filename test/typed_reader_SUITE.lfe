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

;; File    : typed_reader_SUITE.lfe
;; Purpose : End-to-end compile+run tests for reader forms (M9): tuple,
;;           binary, quasiquote — through the full chain.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_reader_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; D-2: Tuple
   (d2_tuple_expression 1)
   (d2_tuple_pattern 1)
   ;; D-3: Binary
   (d3_binary_value 1)
   ;; D-4: Quasiquote
   (d4_quasiquote_unquote 1)
   (d4_quasiquote_splice 1)
   (d4_qq_tuple_pattern_binds 1)
   (d4_qq_tuple_expression 1)
   (p8_plain_defun_qq_tuple 1)))

(defun all ()
  '(d2_tuple_expression
    d2_tuple_pattern
    d3_binary_value
    d4_quasiquote_unquote
    d4_quasiquote_splice
    d4_qq_tuple_pattern_binds
    d4_qq_tuple_expression
    p8_plain_defun_qq_tuple))

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
;;; D-2: Tuple — expression + pattern position
;;; ============================================================

(defun d2_tuple_expression (config)
  (compile-and-load config "reader" "tuple_test.lfet" 'tuple_test)
  (let ((pair (tuple_test:make-pair 'hello 'world)))
    (case pair
      (#(hello world) 'ok)
      (other (ct:fail `#(wrong_pair ,other))))))

(defun d2_tuple_pattern (config)
  (compile-and-load config "reader" "tuple_test.lfet" 'tuple_test)
  (let ((c1 (tuple_test:classify-os #(unix linux)))
        (c2 (tuple_test:classify-os #(unix darwin)))
        (c3 (tuple_test:classify-os #(win32 nt))))
    (case (tuple c1 c2 c3)
      (#(linux macos other) 'ok)
      (other (ct:fail `#(wrong_classify ,other))))))

;;; ============================================================
;;; D-3: Binary — real binary value at runtime
;;; ============================================================

(defun d3_binary_value (config)
  (compile-and-load config "reader" "binary_test.lfet" 'binary_test)
  (let ((g (binary_test:greeting)))
    (case (andalso (is_binary g) (=:= g #"hello"))
      ('true 'ok)
      ('false (ct:fail `#(wrong_binary ,g))))))

;;; ============================================================
;;; D-4: Quasiquote — unquote + splice
;;; ============================================================

(defun d4_quasiquote_unquote (config)
  (compile-and-load config "reader" "quasiquote_test.lfet" 'quasiquote_test)
  (let ((result (quasiquote_test:make-tagged 42)))
    (case result
      ((list 'tagged 42 'done) 'ok)
      (other (ct:fail `#(wrong_tagged ,other))))))

(defun d4_quasiquote_splice (config)
  (compile-and-load config "reader" "quasiquote_test.lfet" 'quasiquote_test)
  (let ((result (quasiquote_test:splice-test '(a b))))
    (case result
      ((list 'start 'a 'b 'end) 'ok)
      (other (ct:fail `#(wrong_splice ,other))))))

;;; ============================================================
;;; D-4: Quasiquoted tuple with unquote (Duncan's exact case)
;;; ============================================================

(defun d4_qq_tuple_pattern_binds (config)
  (compile-and-load config "reader" "qq_tuple_pattern.lfet" 'qq_tuple_pattern)
  (let ((c1 (qq_tuple_pattern:classify #(unix linux)))
        (c2 (qq_tuple_pattern:classify #(unix freebsd)))
        (c3 (qq_tuple_pattern:classify #(win32 nt))))
    (case (tuple c1 c2 c3)
      (#(linux freebsd other) 'ok)
      (other (ct:fail `#(wrong_classify ,other))))))

(defun d4_qq_tuple_expression (config)
  (compile-and-load config "reader" "qq_tuple_expr.lfet" 'qq_tuple_expr)
  (let ((result (qq_tuple_expr:wrap 42)))
    (case result
      (#(ok 42) 'ok)
      (other (ct:fail `#(wrong_wrap ,other))))))

;;; ============================================================
;;; P-8: Fully-plain defun with quasiquoted tuple pattern
;;; ============================================================

(defun p8_plain_defun_qq_tuple (config)
  (compile-and-load config "reader" "plain_defun.lfet" 'plain_defun)
  (let ((c1 (plain_defun:classify #(unix linux)))
        (c2 (plain_defun:classify #(unix freebsd)))
        (c3 (plain_defun:classify #(win32 nt))))
    (case (tuple c1 c2 c3)
      (#(linux freebsd other) 'ok)
      (other (ct:fail `#(wrong_classify ,other))))))

;;; ============================================================
;;; Helpers
;;; ============================================================

(defun compile-and-load (config sub-dir file-name mod-name)
  (let* ((forms (check-and-decode config sub-dir file-name))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms file-name priv-dir)
      (`#(ok ,_mod ,beam-bin)
       (code:purge mod-name)
       (let ((`#(module ,_) (code:load_binary mod-name
                              (++ (atom_to_list mod-name) ".beam")
                              beam-bin)))
         'ok))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

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
