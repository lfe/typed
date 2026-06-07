-module(typed_driver).

-export([compile_forms/3]).

-include_lib("lfe/src/lfe_comp.hrl").

-spec compile_forms([{term(), integer()}], string(), string()) ->
    {ok, module(), binary()} | {error, term()}.
compile_forms(FormLinePairs, SourceFile, OutDir) ->
    Ci = #cinfo{file = SourceFile, opts = [debug_info], ipath = ["."]},
    case lfe_lint:module(FormLinePairs, Ci) of
        {ok, _Name, LintWarns} ->
            lists:foreach(
              fun({Line, Mod, Warn}) ->
                      Msg = Mod:format_error(Warn),
                      io:format("~s:~w: warning: ~s~n", [SourceFile, Line, Msg])
              end, LintWarns),
            compile_codegen(FormLinePairs, Ci, SourceFile, OutDir);
        {error, LintErrors, LintWarns} ->
            lists:foreach(
              fun({Line, Mod, Warn}) ->
                      Msg = Mod:format_error(Warn),
                      io:format("~s:~w: warning: ~s~n", [SourceFile, Line, Msg])
              end, LintWarns),
            lists:foreach(
              fun({Line, Mod, Err}) ->
                      Msg = Mod:format_error(Err),
                      io:format("~s:~w: ~s~n", [SourceFile, Line, Msg])
              end, LintErrors),
            {error, {lint, LintErrors}}
    end.

compile_codegen(FormLinePairs, Ci, SourceFile, OutDir) ->
    case lfe_codegen:module(FormLinePairs, Ci) of
        {ok, ModName, AST, _Warns} ->
            CompOpts = [{source, SourceFile}, return, binary,
                        debug_info, nowarn_unused_vars],
            case compile:forms(AST, CompOpts) of
                {ok, _, Bin, CompWarns} ->
                    report_erl_warnings(SourceFile, CompWarns),
                    BeamFile = filename:join(OutDir,
                                            atom_to_list(ModName) ++ ".beam"),
                    case file:write_file(BeamFile, Bin) of
                        ok -> {ok, ModName, Bin};
                        {error, Reason} -> {error, {write, Reason}}
                    end;
                {error, CompErrors, CompWarns} ->
                    report_erl_warnings(SourceFile, CompWarns),
                    report_erl_errors(SourceFile, CompErrors),
                    {error, {compile, CompErrors}}
            end;
        {error, CgErrors, _CgWarns} ->
            {error, {codegen, CgErrors}}
    end.

report_erl_warnings(_File, []) -> ok;
report_erl_warnings(_File, Ws) ->
    lists:foreach(
      fun({_F, FileWarns}) ->
              lists:foreach(
                fun({Line, Mod, Warn}) ->
                        Msg = Mod:format_error(Warn),
                        io:format("warning: ~w: ~s~n", [Line, Msg])
                end, FileWarns)
      end, Ws).

report_erl_errors(_File, []) -> ok;
report_erl_errors(_File, Es) ->
    lists:foreach(
      fun({_F, FileErrors}) ->
              lists:foreach(
                fun({Line, Mod, Err}) ->
                        Msg = Mod:format_error(Err),
                        io:format("error: ~w: ~s~n", [Line, Msg])
                end, FileErrors)
      end, Es).
