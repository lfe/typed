(defmodule dirs-lin
  (export
   (assemble 1)))

(defun assemble
  (('home) (dirs-common:home))
  (('cache) (env-or-default "XDG_CACHE_HOME" (dirs-common:home-subdir '(".cache"))))
  (('config) (config))
  (('config-local) (config))
  (('data) (data))
  (('data-local) (data))
  (('executable) (env-or-default "XDG_BIN_DIR" (dirs-common:home-subdir '(".local" "bin"))))
  (('preference) (config))
  (('runtime) (env-or-default "XDG_RUNTIME_DIR" 'undefined))
  (('state) (env-or-default "XDG_STATE_DIR" (dirs-common:home-subdir '(".local" "state"))))
  (('audio) (env-or-default "XDG_MUSIC_DIR" 'undefined))
  (('desktop) (env-or-default "XDG_DESKTOP_DIR" 'undefined))
  (('document) (env-or-default "XDG_DOCUMENTS_DIR" 'undefined))
  (('download) (env-or-default "XDG_DOWNLOAD_DIR" 'undefined))
  (('font) (filename:join (list (data) "fonts")))
  (('picture) (env-or-default "XDG_PICTURES_DIR" 'undefined))
  (('public) (env-or-default "XDG_PUBLICSHARE_DIR" 'undefined))
  (('template) (env-or-default "XDG_TEMPLATES_DIR" 'undefined))
  (('video) (env-or-default "XDG_VIDEOS_DIR" 'undefined)))

;;; Private Functions

(defun config ()
  (env-or-default "XDG_CONFIG_HOME" (dirs-common:home-subdir ".config")))

(defun data ()
  (env-or-default "XDG_DATA_HOME" (dirs-common:home-subdir '(".local" "share"))))

(defun env-or-default (env-var default)
  (case (os:getenv env-var)
    ("" default)
    (val (if (dirs-common:abs-path? val)
           val
           default))))
