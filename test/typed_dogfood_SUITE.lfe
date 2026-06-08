;; Copyright (c) 2026 Duncan McGreggor
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; ...
;; File    : typed_dogfood_SUITE.lfe
;; Purpose : M5 dogfood — realistic orders module end-to-end.

(include-lib "lfe/src/lfe_comp.hrl")

(defmodule typed_dogfood_SUITE
  (export
   (all 0)
   (suite 0)
   (groups 0)
   (init_per_suite 1)
   (end_per_suite 1)
   (p1_status_label 1)
   (p1_is_complete 1)
   (p1_line_total 1)
   (p1_apply_discount 1)
   (p1_decode_valid 1)
   (p1_decode_invalid 1)
   (p1_decode_bad_field 1)
   (p6_wrong_type_crashes 1)))

(defun all ()
  '(p1_status_label
    p1_is_complete
    p1_line_total
    p1_apply_discount
    p1_decode_valid
    p1_decode_invalid
    p1_decode_bad_field
    p6_wrong_type_crashes))

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
       (tuple 'skip "typed-check binary not found")))))

(defun end_per_suite (_config) 'ok)

;;; P-1: status-label — all 5 constructors

(defun p1_status_label (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (let ((r1 (call 'orders 'status-label 'pending))
               (r2 (call 'orders 'status-label (tuple 'shipped "TRK42")))
               (r3 (call 'orders 'status-label 'delivered))
               (r4 (call 'orders 'status-label (tuple 'cancelled "bad addr"))))
           (case (tuple r1 r2 r3 r4)
             (#("pending" "shipped: TRK42" "delivered" "cancelled: bad addr") 'ok)
             (other (ct:fail `#(wrong_labels ,other)))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: is-complete

(defun p1_is_complete (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (tuple (call 'orders 'is-complete 'delivered)
                      (call 'orders 'is-complete 'pending)
                      (call 'orders 'is-complete (tuple 'cancelled "x")))
           (#(true false true) 'ok)
           (other (ct:fail `#(wrong_complete ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: line-total

(defun p1_line_total (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (call 'orders 'line-total 3 1500)
           (4500 'ok)
           (other (ct:fail `#(wrong_total ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: apply-discount

(defun p1_apply_discount (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (call 'orders 'apply-discount 1000 20)
           (800 'ok)
           (other (ct:fail `#(wrong_discount ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: decode valid

(defun p1_decode_valid (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (call 'orders 'decode-order-status (tuple 'shipped "ABC"))
           (#(ok #(shipped "ABC")) 'ok)
           (other (ct:fail `#(wrong_decode_valid ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: decode invalid (wrong type entirely)

(defun p1_decode_invalid (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (call 'orders 'decode-order-status 42)
           (`#(error #(type_error ,info))
            (case (maps:get 'expected info)
              ('order-status 'ok)
              (other (ct:fail `#(wrong_expected ,other)))))
           (other (ct:fail `#(should_error ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-1: decode bad field (right tag, wrong field type)

(defun p1_decode_bad_field (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (case (call 'orders 'decode-order-status (tuple 'shipped 999))
           (`#(error #(type_error ,info))
            (case (tuple (maps:get 'expected info) (maps:get 'path info))
              (#(string (tracking)) 'ok)
              (other (ct:fail `#(wrong_bad_field ,other)))))
           (other (ct:fail `#(should_error_field ,other))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

;;; P-6: wrong type crashes with structured error (guard enforcement)

(defun p6_wrong_type_crashes (config)
  (let* ((forms (check-and-decode config "dogfood" "orders.tlfe"))
         (priv-dir (proplists:get_value 'priv_dir config)))
    (case (typed_driver:compile_forms forms "orders.tlfe" priv-dir)
      (`#(ok orders ,beam-bin)
       (code:purge 'orders)
       (let ((`#(module orders) (code:load_binary 'orders "orders.beam" beam-bin)))
         (try (call 'orders 'line-total "three" 1500)
           (catch
             (`#(error #(type_error ,info) ,_)
              (case (maps:get 'expected info)
                ('integer 'ok)
                (other (ct:fail `#(wrong_crash_expected ,other)))))
             (`#(error function_clause ,_)
              (ct:fail '#(got_function_clause)))))))
      (`#(error ,reason) (ct:fail `#(compile_failed ,reason))))))

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
