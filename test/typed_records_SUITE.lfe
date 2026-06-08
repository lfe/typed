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

;; File    : typed_records_SUITE.lfe
;; Purpose : End-to-end tests for defrecord/typed (M6): constructor,
;;           accessors, updaters, type guards, registry, diagnostics.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_records_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   ;; R-2: Constructor
   (r2_make_order 1)
   (r2_make_order_bad_type 1)
   ;; R-3: Accessors
   (r3_accessors 1)
   ;; R-4: Updater
   (r4_set_field 1)
   (r4_set_field_bad_type 1)
   (r4_set_immutable 1)
   ;; R-5: Type integration
   (r5_typed_fun_over_record 1)
   ;; R-6: Registry
   (r6_registry_attr 1)
   ;; R-7: Diagnostics
   (r7_make_arity_mismatch 1)
   ;; R-8: Dogfood end-to-end
   (r8_dogfood_construct_access_update 1)))

(defun all ()
  '(r2_make_order
    r2_make_order_bad_type
    r3_accessors
    r4_set_field
    r4_set_field_bad_type
    r4_set_immutable
    r5_typed_fun_over_record
    r6_registry_attr
    r7_make_arity_mismatch
    r8_dogfood_construct_access_update))

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

;;; ============================================================
;;; R-2: Constructor make-order
;;; ============================================================

(defun r2_make_order (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let ((o (order:make-order 42 'pending 1000)))
    (case o
      (#(order 42 pending 1000) 'ok)
      (other (ct:fail `#(wrong_record ,other))))))

(defun r2_make_order_bad_type (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (try
    (progn
      (order:make-order "not-an-int" 'pending 1000)
      (ct:fail 'should_have_crashed))
    (catch
      (`#(error #(type_error ,err-map) ,_)
       (let ((expected (maps:get 'expected err-map))
             (fn-name  (maps:get 'function err-map)))
         (case (andalso (=:= expected 'order)
                        (=:= fn-name 'make-order))
           ('true 'ok)
           ('false (ct:fail `#(wrong_error ,err-map)))))))))

;;; ============================================================
;;; R-3: Accessors
;;; ============================================================

(defun r3_accessors (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let ((o (order:make-order 42 'pending 1000)))
    (let ((id     (order:order-id o))
          (status (order:order-status o))
          (total  (order:order-total o)))
      (case (andalso (=:= id 42)
                     (=:= status 'pending)
                     (=:= total 1000))
        ('true 'ok)
        ('false (ct:fail `#(wrong_accessors ,id ,status ,total)))))))

;;; ============================================================
;;; R-4: Updater set-order-<field>
;;; ============================================================

(defun r4_set_field (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let* ((o  (order:make-order 42 'pending 1000))
         (o2 (order:set-order-total o 2000)))
    (case o2
      (#(order 42 pending 2000) 'ok)
      (other (ct:fail `#(wrong_updated ,other))))))

(defun r4_set_field_bad_type (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let ((o (order:make-order 42 'pending 1000)))
    (try
      (progn
        (order:set-order-total o "not-an-int")
        (ct:fail 'should_have_crashed))
      (catch
        (`#(error #(type_error ,err-map) ,_)
         (let ((expected (maps:get 'expected err-map))
               (fn-name  (maps:get 'function err-map)))
           (case (andalso (=:= expected 'integer)
                          (=:= fn-name 'set-order-total))
             ('true 'ok)
             ('false (ct:fail `#(wrong_error ,err-map))))))))))

(defun r4_set_immutable (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let* ((o  (order:make-order 42 'pending 1000))
         (_o2 (order:set-order-total o 2000)))
    (case (order:order-total o)
      (1000 'ok)
      (other (ct:fail `#(original_mutated ,other))))))

;;; ============================================================
;;; R-5: Type integration — defun/typed over a record
;;; ============================================================

(defun r5_typed_fun_over_record (config)
  (compile-and-load config "records" "order_ops.tlfe" 'order_ops)
  (let* ((o (order_ops:make-order 1 'pending 500))
         (total (order_ops:get-total o)))
    (case total
      (500 'ok)
      (other (ct:fail `#(wrong_total ,other))))))

;;; ============================================================
;;; R-6: Registry attribute
;;; ============================================================

(defun r6_registry_attr (config)
  (let* ((forms (check-and-decode config "records" "order.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "order.tlfe" priv-dir)
      (`#(ok order ,beam-bin)
       (let ((`#(ok #(order ,chunks))
              (beam_lib:chunks beam-bin '(attributes))))
         (let* ((attrs (proplists:get_value 'attributes chunks))
                (registry (proplists:get_value 'typed-registry attrs)))
           (case registry
             ('undefined (ct:fail '#(no_registry_attr)))
             ((cons (cons 'order _rest) _)
              'ok)
             (other (ct:fail `#(order_not_in_registry ,other)))))))
      (`#(error ,reason)
       (ct:fail `#(compile_failed ,reason))))))

;;; ============================================================
;;; R-7: Diagnostics — make- arity mismatch
;;; ============================================================

(defun r7_make_arity_mismatch (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (try
    (progn
      (order:make-order 42 'pending)
      (ct:fail 'should_have_crashed))
    (catch
      (`#(error #(undef ,_) ,_) 'ok)
      (`#(error undef ,_) 'ok))))

;;; ============================================================
;;; R-8: Dogfood end-to-end
;;; ============================================================

(defun r8_dogfood_construct_access_update (config)
  (compile-and-load config "records" "order.tlfe" 'order)
  (let* ((o1 (order:make-order 1 'pending 500))
         (id (order:order-id o1))
         (o2 (order:set-order-status o1 'shipped))
         (o3 (order:set-order-total o2 750)))
    (case (andalso (=:= id 1)
                   (=:= (order:order-status o2) 'shipped)
                   (=:= (order:order-total o3) 750)
                   (=:= (order:order-total o1) 500))
      ('true 'ok)
      ('false
       (ct:fail `#(dogfood_failed ,o1 ,o2 ,o3))))))

;;; ============================================================
;;; Internal helpers (same pattern as typed_adt_SUITE)
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
