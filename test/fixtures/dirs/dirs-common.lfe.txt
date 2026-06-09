(defmodule dirs-common
  (export
   (abs-path? 1)
   (home 0)
   (home-subdir 1)
   (norm-path 2)))

(defun abs-path?
  ((`(#\/ . ,_)) 'true)
  ((_) 'false))

(defun home ()
  (let ((`#(ok ((,home . ,_) . ,_)) (init:get_argument 'home)))
    home))

(defun home-subdir (path-segs)
  (norm-path (home) path-segs))

(defun norm-path
  ((prefix segs) (when (is_list segs))
   (if (io_lib:printable_unicode_list segs)
     (norm-path prefix (list segs))
     (filename:join (++ (list prefix) (norm-segs segs)))))
  ((prefix seg)
   (norm-path prefix (list (norm-seg seg)))))

(defun norm-segs (segs)
  (norm-segs segs '()))

(defun norm-segs
  (('() acc)
   acc)
  ((`(,seg . ,rest) acc)
   (norm-segs rest (++ acc (list (norm-seg seg))))))

(defun norm-seg
  ((seg) (when (is_list seg))
   seg)
  ((seg) (when (is_atom seg))
   (atom_to_list seg))
  ((seg) (when (is_binary seg))
   (binary_to_list seg))
  ((seg)
   (io_lib:format "~p" (list seg))))
