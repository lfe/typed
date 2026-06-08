;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0

;; File    : typed_runtime_SUITE.lfe
;; Purpose : End-to-end tests for M4 runtime enforcement: head guards,
;;           structured type-errors, and the crash-vs-return duality.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_runtime_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; M4-1/M4-2: guards work — correct call succeeds
   (m4_1_correct_call_passes 1)
   ;; M4-3: wrong arg raises structured type-error
   (m4_3_wrong_arg_crashes 1)
   ;; M4-4: structured error has expected fields
   (m4_4_structured_error_fields 1)
   ;; M4-2: wrong-tagged tuple rejected by ADT guard
   (m4_2_wrong_tag_rejected 1)))

(defun all ()
  '(m4_1_correct_call_passes
    m4_3_wrong_arg_crashes
    m4_4_structured_error_fields
    m4_2_wrong_tag_rejected))

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

;;; M4-1/M4-2: correct call passes through guards

(defun m4_1_correct_call_passes (config)
  (let* ((forms (check-and-decode config "runtime" "guarded.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "guarded.tlfe" priv-dir)
      (`#(ok guarded ,beam-bin)
       (code:purge 'guarded)
       (let ((`#(module guarded) (code:load_binary 'guarded "guarded.beam" beam-bin)))
         (let ((result (call 'guarded 'double 21)))
           (case result
             (42 'ok)
             (other (ct:fail `#(wrong_result ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M4-3: wrong-typed arg raises structured type-error (not function_clause)

(defun m4_3_wrong_arg_crashes (config)
  (let* ((forms (check-and-decode config "runtime" "guarded.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "guarded.tlfe" priv-dir)
      (`#(ok guarded ,beam-bin)
       (code:purge 'guarded)
       (let ((`#(module guarded) (code:load_binary 'guarded "guarded.beam" beam-bin)))
         (try (call 'guarded 'double "not-an-integer")
           (catch
             (`#(error #(type_error ,info) ,_stacktrace)
              ;; Structured type-error raised — this is the M4 headline
              (let ((expected (proplists:get_value 'expected info))
                    (function (proplists:get_value 'function info)))
                (case (tuple expected function)
                  (#(integer double) 'ok)
                  (other (ct:fail `#(wrong_error_fields ,other))))))
             (`#(error function_clause ,_)
              (ct:fail '#(got_function_clause_not_type_error)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M4-4: structured error term has all expected fields

(defun m4_4_structured_error_fields (config)
  (let* ((forms (check-and-decode config "runtime" "guarded.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "guarded.tlfe" priv-dir)
      (`#(ok guarded ,beam-bin)
       (code:purge 'guarded)
       (let ((`#(module guarded) (code:load_binary 'guarded "guarded.beam" beam-bin)))
         (try (call 'guarded 'double "oops")
           (catch
             (`#(error #(type_error ,info) ,_)
              (let ((expected (proplists:get_value 'expected info))
                    (got (proplists:get_value 'got info))
                    (function (proplists:get_value 'function info))
                    (arg (proplists:get_value 'arg info)))
                (case (tuple expected function arg)
                  (#(integer double 1)
                   ;; 'got' should be the actual bad value
                   (case got
                     ("oops" 'ok)
                     (other (ct:fail `#(wrong_got_value ,other)))))
                  (other (ct:fail `#(wrong_fields ,other))))))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; M4-2: wrong-tagged tuple rejected by ADT head guard

(defun m4_2_wrong_tag_rejected (config)
  (let* ((forms (check-and-decode config "typecheck/readme" "describe_good.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "describe_good.tlfe" priv-dir)
      (`#(ok describe_good ,beam-bin)
       (code:purge 'describe_good)
       (let ((`#(module describe_good)
              (code:load_binary 'describe_good "describe_good.beam" beam-bin)))
         ;; Correct value passes
         (let ((r (call 'describe_good 'describe 'pending)))
           (case r
             ("queued" 'ok)
             (other (ct:fail `#(correct_value_failed ,other)))))
         ;; Wrong-tagged tuple raises structured type-error
         (try (call 'describe_good 'describe (tuple 'bogus 1))
           (catch
             (`#(error #(type_error ,info) ,_)
              (let ((expected (proplists:get_value 'expected info))
                    (function (proplists:get_value 'function info)))
                (case (tuple expected function)
                  (#(order-status describe) 'ok)
                  (other (ct:fail `#(wrong_error_fields ,other))))))
             (`#(error function_clause ,_)
              (ct:fail '#(got_function_clause_not_type_error)))))))
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
