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

;; File    : typed_cross_module_SUITE.lfe
;; Purpose : End-to-end tests for cross-module type references (M7):
;;           qualified mod:type, import-types, static diagnostics.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_cross_module_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; X-2/X-3: Qualified cross-module type
   (x3_qualified_cross_module 1)
   (x3a_boundary_rejects_non_tuple 1)
   (x3a_boundary_rejects_wrong_tag 1)
   (x3b_cross_module_accessor 1)
   ;; X-4: import-types
   (x4_import_types 1)
   ;; X-5: Static diagnostics
   (x5_unknown_module 1)
   (x5_unknown_type 1)
   (x5_bad_import 1)
   ;; X-6: Dogfood
   (x6_dogfood_cross_module 1)))

(defun all ()
  '(x3_qualified_cross_module
    x3a_boundary_rejects_non_tuple
    x3a_boundary_rejects_wrong_tag
    x3b_cross_module_accessor
    x4_import_types
    x5_unknown_module
    x5_unknown_type
    x5_bad_import
    x6_dogfood_cross_module))

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
;;; X-3: Qualified mod:type resolves across modules
;;; ============================================================

(defun x3_qualified_cross_module (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         ;; First compile the producer (orders)
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         ;; Now compile the consumer (orders_web) — uses orders:order
         (let ((web-forms (check-and-decode config "cross_module" "orders_web.tlfe")))
           (case (typed_driver:compile_forms web-forms "orders_web.tlfe" priv-dir)
             (`#(ok orders_web ,web-bin)
              (code:purge 'orders_web)
              (let ((`#(module orders_web)
                     (code:load_binary 'orders_web "orders_web.beam" web-bin)))
                (let* ((o (orders:make-order 42 'pending 1000))
                       (total (orders_web:get-order-total o)))
                  (case total
                    (1000 'ok)
                    (other (ct:fail `#(wrong_total ,other)))))))
             (`#(error ,reason)
              (ct:fail `#(web_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; X-3a: Boundary rejection — non-tuple
;;; ============================================================

(defun x3a_boundary_rejects_non_tuple (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         (let ((web-forms (check-and-decode config "cross_module" "orders_web.tlfe")))
           (case (typed_driver:compile_forms web-forms "orders_web.tlfe" priv-dir)
             (`#(ok orders_web ,web-bin)
              (code:purge 'orders_web)
              (let ((`#(module orders_web)
                     (code:load_binary 'orders_web "orders_web.beam" web-bin)))
                (try
                  (progn
                    (orders_web:get-order-total 42)
                    (ct:fail 'should_have_crashed))
                  (catch
                    (`#(error #(type_error ,err-map) ,_)
                     (let ((expected (maps:get 'expected err-map))
                           (fn-name  (maps:get 'function err-map)))
                       (case (tuple expected fn-name)
                         (#(orders:order get-order-total) 'ok)
                         (other (ct:fail `#(wrong_error ,other ,err-map))))))))))
             (`#(error ,reason)
              (ct:fail `#(web_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; X-3a: Boundary rejection — wrong-tagged tuple
;;; ============================================================

(defun x3a_boundary_rejects_wrong_tag (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         (let ((web-forms (check-and-decode config "cross_module" "orders_web.tlfe")))
           (case (typed_driver:compile_forms web-forms "orders_web.tlfe" priv-dir)
             (`#(ok orders_web ,web-bin)
              (code:purge 'orders_web)
              (let ((`#(module orders_web)
                     (code:load_binary 'orders_web "orders_web.beam" web-bin)))
                (try
                  (progn
                    (orders_web:get-order-total (tuple 'not_order 1 2 3))
                    (ct:fail 'should_have_crashed))
                  (catch
                    (`#(error #(type_error ,err-map) ,_)
                     (let ((expected (maps:get 'expected err-map))
                           (got      (maps:get 'got err-map)))
                       (case expected
                         ('orders:order
                          (case got
                            (#(not_order 1 2 3) 'ok)
                            (other (ct:fail `#(wrong_got ,other)))))
                         (other (ct:fail `#(wrong_expected ,other ,err-map))))))))))
             (`#(error ,reason)
              (ct:fail `#(web_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; X-3b: Cross-module accessor call
;;; ============================================================

(defun x3b_cross_module_accessor (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         (let ((acc-forms (check-and-decode config "cross_module" "orders_web_accessor.tlfe")))
           (case (typed_driver:compile_forms acc-forms "orders_web_accessor.tlfe" priv-dir)
             (`#(ok orders_web_accessor ,acc-bin)
              (code:purge 'orders_web_accessor)
              (let ((`#(module orders_web_accessor)
                     (code:load_binary 'orders_web_accessor "orders_web_accessor.beam" acc-bin)))
                (let* ((o (orders:make-order 77 'shipped 3500))
                       (total (orders_web_accessor:get-order-total o)))
                  (case total
                    (3500 'ok)
                    (other (ct:fail `#(wrong_accessor_result ,other)))))))
             (`#(error ,reason)
              (ct:fail `#(accessor_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; X-4: import-types resolves bare names
;;; ============================================================

(defun x4_import_types (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         (let ((import-forms (check-and-decode config "cross_module" "orders_web_import.tlfe")))
           (case (typed_driver:compile_forms import-forms "orders_web_import.tlfe" priv-dir)
             (`#(ok orders_web_import ,import-bin)
              (code:purge 'orders_web_import)
              (let ((`#(module orders_web_import)
                     (code:load_binary 'orders_web_import "orders_web_import.beam" import-bin)))
                (let* ((o (orders:make-order 7 'shipped 500))
                       (total (orders_web_import:get-order-total o)))
                  (case total
                    (500 'ok)
                    (other (ct:fail `#(wrong_total ,other)))))))
             (`#(error ,reason)
              (ct:fail `#(import_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; X-5: Static diagnostics
;;; ============================================================

(defun x5_unknown_module (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (fixture (filename:join (list fixture-dir "cross_module" "bad_unknown_module.tlfe")))
         (`#(,exit-code ,output) (run-checker-raw checker-bin fixture)))
    (case exit-code
      (0 (ct:fail '#(expected_nonzero_exit)))
      (_
       (case (string:find output "unknown module `bogus`")
         ('nomatch (ct:fail `#(wrong_diagnostic ,output)))
         (_ 'ok))))))

(defun x5_unknown_type (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (fixture (filename:join (list fixture-dir "cross_module" "bad_unknown_type.tlfe")))
         (`#(,exit-code ,output) (run-checker-raw checker-bin fixture)))
    (case exit-code
      (0 (ct:fail '#(expected_nonzero_exit)))
      (_
       (case (string:find output "unknown type `nonexistent` in module `orders`")
         ('nomatch (ct:fail `#(wrong_diagnostic ,output)))
         (_ 'ok))))))

(defun x5_bad_import (config)
  (let* ((checker-bin (proplists:get_value 'checker_bin config))
         (fixture-dir (proplists:get_value 'fixture_dir config))
         (fixture (filename:join (list fixture-dir "cross_module" "bad_import.tlfe")))
         (`#(,exit-code ,output) (run-checker-raw checker-bin fixture)))
    (case exit-code
      (0 (ct:fail '#(expected_nonzero_exit)))
      (_
       (case (string:find output "unknown type `nonexistent` in module `orders`")
         ('nomatch (ct:fail `#(wrong_diagnostic ,output)))
         (_ 'ok))))))

;;; ============================================================
;;; X-6: Dogfood — two-module order domain
;;; ============================================================

(defun x6_dogfood_cross_module (config)
  (let* ((priv-dir (proplists:get_value 'priv_dir config))
         (orders-forms (check-and-decode config "cross_module" "orders.tlfe")))
    (case (typed_driver:compile_forms orders-forms "orders.tlfe" priv-dir)
      (`#(ok orders ,orders-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" orders-bin)))
         ;; Build the web module using qualified refs
         (let ((web-forms (check-and-decode config "cross_module" "orders_web.tlfe")))
           (case (typed_driver:compile_forms web-forms "orders_web.tlfe" priv-dir)
             (`#(ok orders_web ,web-bin)
              (code:purge 'orders_web)
              (let ((`#(module orders_web)
                     (code:load_binary 'orders_web "orders_web.beam" web-bin)))
                ;; End-to-end: create in orders, consume in orders_web
                (let* ((o (orders:make-order 99 'delivered 2500))
                       (total (orders_web:get-order-total o)))
                  (case (andalso (=:= total 2500) (=:= (orders:order-id o) 99))
                    ('true 'ok)
                    ('false (ct:fail `#(dogfood_failed ,total ,o)))))))
             (`#(error ,reason)
              (ct:fail `#(web_compile_failed ,reason)))))))
      (`#(error ,reason)
       (ct:fail `#(orders_compile_failed ,reason))))))

;;; ============================================================
;;; Internal helpers
;;; ============================================================

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
