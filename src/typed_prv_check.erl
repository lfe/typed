-module(typed_prv_check).

-export([init/1, do/1, format_error/1]).

-behaviour(provider).

-define(PROVIDER, check).
-define(NAMESPACE, typed).
-define(DEPS, [compile]).

init(State) ->
    Provider = providers:create([
        {name, ?PROVIDER},
        {module, ?MODULE},
        {namespace, ?NAMESPACE},
        {bare, true},
        {deps, ?DEPS},
        {example, "rebar3 typed check"},
        {opts, []},
        {short_desc, "Run the typed-check shape checker on typed LFE sources"},
        {desc, "Parses defun/typed forms, validates shape, lowers to "
               "plain LFE, and compiles to .beam with original-source "
               "line injection."}
    ]),
    {ok, rebar_state:add_provider(State, Provider)}.

do(State) ->
    rebar_api:info("Running typed check ...", []),
    AppInfos = case rebar_state:current_app(State) of
                   undefined -> rebar_state:project_apps(State);
                   AppInfo -> [AppInfo]
               end,
    CheckerBin = find_checker_binary(State),
    case CheckerBin of
        {error, not_found} ->
            rebar_api:abort("typed-check binary not found; run "
                            "'cargo build' in checker/", []);
        {ok, Bin} ->
            lists:foreach(
              fun(AppInfo) -> check_app(AppInfo, Bin, State) end,
              AppInfos),
            {ok, State}
    end.

format_error(Reason) ->
    io_lib:format("~p", [Reason]).

find_checker_binary(State) ->
    Dir = rebar_state:dir(State),
    Candidates = [
        filename:join([Dir, "checker", "target", "debug", "typed-check"]),
        filename:join([Dir, "checker", "target", "release", "typed-check"])
    ],
    case lists:filter(fun filelib:is_file/1, Candidates) of
        [Bin | _] -> {ok, Bin};
        [] -> {error, not_found}
    end.

check_app(AppInfo, CheckerBin, _State) ->
    SrcDirs = rebar_app_info:get(AppInfo, src_dirs, ["src"]),
    AppDir = rebar_app_info:dir(AppInfo),
    OutDir = rebar_app_info:ebin_dir(AppInfo),
    LfeFiles = lists:flatmap(
        fun(SrcDir) ->
                Dir = filename:join(AppDir, SrcDir),
                filelib:wildcard(filename:join(Dir, "*.lfe"))
        end, SrcDirs),
    TypedFiles = lists:filter(fun has_typed_forms/1, LfeFiles),
    lists:foreach(
      fun(File) -> check_file(File, CheckerBin, OutDir) end,
      TypedFiles).

has_typed_forms(File) ->
    case file:read_file(File) of
        {ok, Bin} -> binary:match(Bin, <<"defun/typed">>) =/= nomatch;
        _ -> false
    end.

check_file(File, CheckerBin, OutDir) ->
    rebar_api:info("  checking ~s", [File]),
    EetfFile = File ++ ".eetf",
    Cmd = lists:flatten(io_lib:format("\"~s\" \"~s\" --output \"~s\"",
                                       [CheckerBin, File, EetfFile])),
    case os:cmd(Cmd ++ " ; echo $?") of
        Output ->
            Lines = string:split(string:trim(Output), "\n", all),
            ExitStr = lists:last(Lines),
            Diags = lists:droplast(Lines),
            case ExitStr of
                "0" ->
                    lists:foreach(
                      fun(D) -> rebar_api:info("  ~s", [D]) end,
                      Diags),
                    load_and_compile(File, EetfFile, OutDir);
                _ ->
                    lists:foreach(
                      fun(D) -> rebar_api:error("  ~s", [D]) end,
                      Diags),
                    file:delete(EetfFile),
                    rebar_api:abort("typed check failed for ~s", [File])
            end
    end.

load_and_compile(SourceFile, EetfFile, OutDir) ->
    case file:read_file(EetfFile) of
        {ok, Bin} ->
            file:delete(EetfFile),
            FormLinePairs = binary_to_term(Bin),
            case typed_driver:compile_forms(FormLinePairs,
                                            filename:basename(SourceFile),
                                            OutDir) of
                {ok, Mod, _Beam} ->
                    rebar_api:info("  compiled ~p", [Mod]);
                {error, Reason} ->
                    rebar_api:abort("compile failed for ~s: ~p",
                                   [SourceFile, Reason])
            end;
        {error, Reason} ->
            rebar_api:abort("failed to read ~s: ~p", [EetfFile, Reason])
    end.
