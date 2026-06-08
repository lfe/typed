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

;; File    : typed_chain_SUITE.lfe
;; Purpose : End-to-end tests for the typed-check → EETF → driver → BEAM chain.
;;           Verifies line injection (F-8/F-9), EETF round-trip (F-6),
;;           compile+call (F-7), and checker gating (F-10).

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_chain_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   (f6_eetf_roundtrip 1)
   (f7_compile_and_call 1)
   (f8_runtime_line_injection 1)
   (f9_compile_error_line_injection 1)
   (f9b_compile_error_file_injection 1)
   (f10_checker_gates_malformed 1)))

(defun all ()
  '(f6_eetf_roundtrip
    f7_compile_and_call
    f8_runtime_line_injection
    f9_compile_error_line_injection
    f9b_compile_error_file_injection
    f10_checker_gates_malformed))

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

(defun end_per_suite (config) 'ok)

;;; F-6: EETF round-trip

(defun f6_eetf_roundtrip (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "good" "hello.lfet")))
         (eetf-file   (filename:join priv-dir "hello.eetf"))
         (`#(0 ,_)    (run-checker checker-bin fixture eetf-file))
         (`#(ok ,bin)  (file:read_file eetf-file))
         (forms       (binary_to_term bin)))
    (let ((`(#(,mod-form ,_mod-line) #(,func-form ,func-line) . ,_) forms))
      (let ((`(define-module hello . ,_) mod-form))
        (let ((`(define-function greet . ,_) func-form))
          (case func-line
            (35 'ok)
            (other (ct:fail `#(wrong_func_line ,other)))))))))

;;; F-7: Compile and call

(defun f7_compile_and_call (config)
  (let* ((forms    (check-and-decode config "good" "hello.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "hello.lfe" priv-dir)
      (`#(ok hello ,beam-bin)
       (let ((`#(module hello) (code:load_binary 'hello "hello.beam" beam-bin)))
         (case (hello:greet #"world")
           (`("Hello " #"world") 'ok)
           (other (ct:fail `#(wrong_result ,other))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; F-8 HEADLINE: Runtime line injection

(defun f8_runtime_line_injection (config)
  (let* ((forms    (check-and-decode config "crash" "boom.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "boom.lfe" priv-dir)
      (`#(ok boom ,beam-bin)
       (code:purge 'boom)
       (let ((`#(module boom) (code:load_binary 'boom "boom.beam" beam-bin)))
         (try (boom:kaboom)
           (catch
             (`#(error bang ,stacktrace)
              (let ((`(#(boom kaboom 0 ,info) . ,_) stacktrace))
                (let ((file (proplists:get_value 'file info))
                      (line (proplists:get_value 'line info)))
                  (case (tuple file line)
                    (#("boom.lfe" 42) 'ok)
                    (other (ct:fail `#(wrong_file_line ,other)))))))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; F-9: Compile error line injection (lfe_lint path)

(defun f9_compile_error_line_injection (config)
  (let* ((forms    (check-and-decode config "comperr" "unbound.lfet"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "unbound.lfe" priv-dir)
      (`#(error #(lint (#(71 lfe_lint #(unbound_symbol totally_unbound_var)))))
       'ok)
      (other
       (ct:fail `#(unexpected_result ,other))))))

;;; F-9b: Compile error file+line injection (compile:forms/erlc path)

(defun f9b_compile_error_file_injection (_config)
  (let* ((orig-file "injected_origin.lfet")
         (injected-line 9042)
         (forms (list
                 (tuple '(define-module f9bmod () ((export (ok_fn 0)))) 1)
                 (tuple '(define-function ok_fn () (lambda () (quote ok)))
                        injected-line)))
         (ci (make-cinfo file orig-file
                         opts '(debug_info)
                         ipath '("."))))
    (case (lfe_codegen:module forms ci)
      (`#(ok f9bmod ,ast ,_warns)
       (let* ((bad-func
               `#(function ,injected-line bad_fn 1
                  (#(clause ,injected-line
                     (#(var ,injected-line X))
                     ()
                     (#(var ,injected-line Unbound)
                      #(var ,injected-line X))))))
              (ast1 (++ ast (list bad-func)))
              (comp-opts `(#(source ,orig-file) return binary debug_info)))
         (case (compile:forms ast1 comp-opts)
           (`#(error (#(,file ,file-errors)) ,_warnings)
            (let ((found (lists:keyfind injected-line 1 file-errors)))
              (if (and (== file orig-file)
                       (== found (tuple injected-line 'erl_lint
                                        (tuple 'unbound_var 'Unbound))))
                'ok
                (ct:fail `#(wrong_file_or_error #(file ,file) #(found ,found))))))
           (other
            (ct:fail `#(expected_compile_error ,other))))))
      (other
       (ct:fail `#(codegen_failed ,other))))))

;;; F-10: Checker gates on malformed input

(defun f10_checker_gates_malformed (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (priv-dir    (proplists:get_value 'priv_dir config))
         (fixture     (filename:join (list fixture-dir "malformed" "bad.lfet")))
         (eetf-file   (filename:join priv-dir "bad.eetf"))
         (`#(,exit-code ,output) (run-checker checker-bin fixture eetf-file)))
    (case (/= exit-code 0)
      ('true
       (case (string:find output "17:1")
         ('nomatch (ct:fail `#(diagnostic_missing_span ,output)))
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
                       (++ sub-dir "_" file-name ".eetf")))
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
