;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0

;; File    : typed_typecheck_SUITE.lfe
;; Purpose : End-to-end tests for M3 contract checking: body-vs-returns,
;;           call-arg checking, and dynamic boundary.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_typecheck_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; M3-3: body vs :returns
   (m3_3_well_typed_passes 1)
   (m3_3_return_mismatch_rejected 1)
   ;; M3-4: call arg mismatch
   (m3_4_arg_mismatch_rejected 1)
   ;; C-4: README describe demo
   (c4_describe_good_passes 1)
   (c4_describe_bad_rejected 1)))

(defun all ()
  '(m3_3_well_typed_passes
    m3_3_return_mismatch_rejected
    m3_4_arg_mismatch_rejected
    c4_describe_good_passes
    c4_describe_bad_rejected))

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

;;; M3-3: well-typed function compiles and runs

(defun m3_3_well_typed_passes (config)
  (let* ((forms (check-and-decode config "typecheck/good" "well_typed.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "well_typed.lfet" priv-dir)
      (`#(ok well_typed ,beam-bin)
       (code:purge 'well_typed)
       (let ((`#(module well_typed) (code:load_binary 'well_typed "well_typed.beam" beam-bin)))
         (let ((result (call 'well_typed 'double 21)))
           (case result
             (42 'ok)
             (other (ct:fail `#(wrong_result ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M3-3: return type mismatch rejected by checker

(defun m3_3_return_mismatch_rejected (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "typecheck" "bad" "return_mismatch.lfet")))
         (eetf-file   (filename:join priv-dir "return_mismatch.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       (case (andalso (=/= 'nomatch (string:find output "integer"))
                      (=/= 'nomatch (string:find output "binary")))
         ('true 'ok)
         ('false (ct:fail `#(missing_type_names ,output)))))
      ('false
       (ct:fail '#(checker_should_have_rejected))))))

;;; M3-4: call argument type mismatch rejected

(defun m3_4_arg_mismatch_rejected (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "typecheck" "bad" "arg_mismatch.lfet")))
         (eetf-file   (filename:join priv-dir "arg_mismatch.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       (case (=/= 'nomatch (string:find output "expected type"))
         ('true 'ok)
         ('false (ct:fail `#(missing_expected_type ,output)))))
      ('false
       (ct:fail '#(checker_should_have_rejected))))))

;;; C-4: README describe demo — correct version passes

(defun c4_describe_good_passes (config)
  (let* ((forms (check-and-decode config "typecheck/readme" "describe_good.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "describe_good.lfet" priv-dir)
      (`#(ok describe_good ,beam-bin)
       (code:purge 'describe_good)
       (let ((`#(module describe_good)
              (code:load_binary 'describe_good "describe_good.beam" beam-bin)))
         (let ((r1 (call 'describe_good 'describe 'pending))
               (r2 (call 'describe_good 'describe (tuple 'shipped "TRK123")))
               (r3 (call 'describe_good 'describe (tuple 'cancelled "out of stock"))))
           (case r1
             ("queued"
              (case r3
                ("cancelled: out of stock" 'ok)
                (other (ct:fail `#(wrong_cancelled ,other)))))
             (other (ct:fail `#(wrong_pending ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; C-4: README describe demo — wrong version rejected

(defun c4_describe_bad_rejected (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "typecheck" "readme" "describe_bad.lfet")))
         (eetf-file   (filename:join priv-dir "describe_bad.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       (case (=/= 'nomatch (string:find output "pattern"))
         ('true 'ok)
         ('false (ct:fail `#(missing_type_info ,output)))))
      ('false
       (ct:fail '#(checker_should_have_rejected))))))

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
