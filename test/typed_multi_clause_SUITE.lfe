;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License")

;; File    : typed_multi_clause_SUITE.lfe
;; Purpose : End-to-end tests for multi-clause defun/typed (M11):
;;           value dispatch, type dispatch, pattern+type.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_multi_clause_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; SF-8: Ackermann (value dispatch)
   (sf8_ackermann 1)
   ;; SF-8: Type dispatch
   (sf8_type_dispatch 1)
   ;; SF-5: Wrong-typed arg rejected
   (sf5_wrong_type_rejected 1)
   ;; SF-4: Heterogeneous-return diagnostic
   (sf4_hetero_return_rejected 1)))

(defun all ()
  '(sf8_ackermann
    sf8_type_dispatch
    sf5_wrong_type_rejected
    sf4_hetero_return_rejected))

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
;;; SF-8: Ackermann (value dispatch) — exact results
;;; ============================================================

(defun sf8_ackermann (config)
  (compile-and-load config "multi_clause" "ackermann.lfet" 'ackermann)
  (let ((r1 (ackermann:ack 0 0))
        (r2 (ackermann:ack 1 1))
        (r3 (ackermann:ack 2 2))
        (r4 (ackermann:ack 3 3)))
    (case (tuple r1 r2 r3 r4)
      (#(1 3 7 61) 'ok)
      (other (ct:fail `#(wrong_ackermann ,other))))))

;;; ============================================================
;;; SF-8: Type dispatch — norm-seg style
;;; ============================================================

(defun sf8_type_dispatch (config)
  (compile-and-load config "multi_clause" "norm_seg.lfet" 'norm_seg)
  (let ((s (norm_seg:to-string "hello"))
        (i (norm_seg:to-string 42))
        (a (norm_seg:to-string 'world)))
    (case (tuple s i a)
      (#("hello" "42" "world") 'ok)
      (other (ct:fail `#(wrong_dispatch ,other))))))

;;; ============================================================
;;; SF-5: Wrong-typed arg → structured type-error
;;; ============================================================

(defun sf5_wrong_type_rejected (config)
  (compile-and-load config "multi_clause" "norm_seg.lfet" 'norm_seg)
  (try
    (progn
      (norm_seg:to-string #(not a string))
      (ct:fail 'should_have_crashed))
    (catch
      (`#(error #(type_error ,err-map) ,_)
       (let ((expected (maps:get 'expected err-map)))
         (case expected
           ('string 'ok)
           (other (ct:fail `#(wrong_expected ,other ,err-map)))))))))

;;; ============================================================
;;; SF-4: Heterogeneous return types → teaching diagnostic
;;; ============================================================

(defun sf4_hetero_return_rejected (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (fixture (filename:join (list fixture-dir "multi_clause" "bad_hetero_return.lfet")))
         (`#(,exit-code ,output) (run-checker-raw checker-bin fixture)))
    (case exit-code
      (0 (ct:fail '#(expected_nonzero_exit)))
      (_
       (case (string:find output "heterogeneous-return overloading not yet supported")
         ('nomatch (ct:fail `#(wrong_diagnostic ,output)))
         (_ 'ok))))))

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

(defun run-checker-raw (checker-bin input-file)
  (let* ((cmd (lists:flatten
               (io_lib:format "\"~s\" \"~s\" 2>&1; echo \"EXIT:$?\""
                              (list checker-bin input-file))))
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
