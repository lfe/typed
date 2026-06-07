-module(typed_chain_SUITE).

-include_lib("common_test/include/ct.hrl").
-include_lib("lfe/src/lfe_comp.hrl").

-export([all/0, suite/0, groups/0,
         init_per_suite/1, end_per_suite/1]).
-export([f6_eetf_roundtrip/1,
         f7_compile_and_call/1,
         f8_runtime_line_injection/1,
         f9_compile_error_line_injection/1,
         f9b_compile_error_file_injection/1,
         f10_checker_gates_malformed/1]).

all() ->
    [f6_eetf_roundtrip,
     f7_compile_and_call,
     f8_runtime_line_injection,
     f9_compile_error_line_injection,
     f9b_compile_error_file_injection,
     f10_checker_gates_malformed].

suite() ->
    [{timetrap, {seconds, 60}}].

groups() ->
    [].

init_per_suite(Config) ->
    ProjectRoot = find_project_root(),
    CheckerBin = filename:join([ProjectRoot, "checker", "target", "debug", "typed-check"]),
    case filelib:is_file(CheckerBin) of
        true ->
            FixtureDir = filename:join(ProjectRoot, "test/fixtures"),
            [{checker_bin, CheckerBin},
             {fixture_dir, FixtureDir},
             {project_root, ProjectRoot} | Config];
        false ->
            {skip, "typed-check binary not found; run 'cargo build' in checker/"}
    end.

end_per_suite(_Config) ->
    ok.

f6_eetf_roundtrip(Config) ->
    CheckerBin = ?config(checker_bin, Config),
    FixtureDir = ?config(fixture_dir, Config),
    PrivDir = ?config(priv_dir, Config),
    Fixture = filename:join([FixtureDir, "good", "hello.tlfe"]),
    EetfFile = filename:join(PrivDir, "hello.eetf"),
    {0, _} = run_checker(CheckerBin, Fixture, EetfFile),
    {ok, Bin} = file:read_file(EetfFile),
    Forms = binary_to_term(Bin),
    [{ModForm, _ModLine}, {FuncForm, FuncLine} | _] = Forms,
    ['define-module', hello | _] = ModForm,
    ['define-function', greet | _] = FuncForm,
    35 = FuncLine,
    ok.

f7_compile_and_call(Config) ->
    Forms = check_and_decode(Config, "good", "hello.tlfe"),
    PrivDir = ?config(priv_dir, Config),
    {ok, hello, BeamBin} = typed_driver:compile_forms(Forms, "hello.lfe", PrivDir),
    {module, hello} = code:load_binary(hello, "hello.beam", BeamBin),
    Result = hello:greet(<<"world">>),
    [<<"Hello ">>, <<"world">>] = Result,
    ok.

f8_runtime_line_injection(Config) ->
    Forms = check_and_decode(Config, "crash", "boom.tlfe"),
    PrivDir = ?config(priv_dir, Config),
    {ok, boom, BeamBin} = typed_driver:compile_forms(Forms, "boom.lfe", PrivDir),
    code:purge(boom),
    {module, boom} = code:load_binary(boom, "boom.beam", BeamBin),
    try boom:kaboom() of
        _ -> ct:fail(should_have_crashed)
    catch
        error:bang:Stacktrace ->
            [{boom, kaboom, 0, Info} | _] = Stacktrace,
            "boom.lfe" = proplists:get_value(file, Info),
            42 = proplists:get_value(line, Info),
            ok
    end.

f9_compile_error_line_injection(Config) ->
    Forms = check_and_decode(Config, "comperr", "unbound.tlfe"),
    PrivDir = ?config(priv_dir, Config),
    {error, {lint, Errors}} = typed_driver:compile_forms(Forms, "unbound.lfe", PrivDir),
    [{71, lfe_lint, {unbound_symbol, totally_unbound_var}}] = Errors,
    ok.

f9b_compile_error_file_injection(_Config) ->
    OrigFile = "injected_origin.tlfe",
    InjectedLine = 9042,
    Forms = [
        {['define-module', f9bmod, [], [[export, [ok_fn, 0]]]], 1},
        {['define-function', ok_fn, [],
          [lambda, [], [quote, ok]]], InjectedLine}
    ],
    Ci = #cinfo{file = OrigFile, opts = [debug_info], ipath = ["."]},
    {ok, f9bmod, AST0, _} = lfe_codegen:module(Forms, Ci),
    BadFunc = {function, InjectedLine, bad_fn, 1,
               [{clause, InjectedLine,
                 [{var, InjectedLine, 'X'}], [],
                 [{var, InjectedLine, 'Unbound'},
                  {var, InjectedLine, 'X'}]}]},
    AST1 = AST0 ++ [BadFunc],
    CompOpts = [{source, OrigFile}, return, binary, debug_info],
    {error, Errors, _Warnings} = compile:forms(AST1, CompOpts),
    [{OrigFile, FileErrors}] = Errors,
    {InjectedLine, erl_lint, {unbound_var, 'Unbound'}} =
        lists:keyfind(InjectedLine, 1, FileErrors),
    ok.

f10_checker_gates_malformed(Config) ->
    CheckerBin = ?config(checker_bin, Config),
    FixtureDir = ?config(fixture_dir, Config),
    PrivDir = ?config(priv_dir, Config),
    Fixture = filename:join([FixtureDir, "malformed", "bad.tlfe"]),
    EetfFile = filename:join(PrivDir, "bad.eetf"),
    {ExitCode, Output} = run_checker(CheckerBin, Fixture, EetfFile),
    true = (ExitCode =/= 0),
    true = (nomatch =/= string:find(Output, "17:1")),
    ok.

%%% --- helpers ---

check_and_decode(Config, SubDir, FileName) ->
    CheckerBin = ?config(checker_bin, Config),
    FixtureDir = ?config(fixture_dir, Config),
    PrivDir = ?config(priv_dir, Config),
    Fixture = filename:join([FixtureDir, SubDir, FileName]),
    EetfFile = filename:join(PrivDir, SubDir ++ "_" ++ FileName ++ ".eetf"),
    {0, _} = run_checker(CheckerBin, Fixture, EetfFile),
    {ok, Bin} = file:read_file(EetfFile),
    binary_to_term(Bin).

run_checker(CheckerBin, InputFile, OutputFile) ->
    Cmd = lists:flatten(io_lib:format(
            "\"~s\" \"~s\" --output \"~s\" 2>&1; echo \"EXIT:$?\"",
            [CheckerBin, InputFile, OutputFile])),
    RawOutput = os:cmd(Cmd),
    Lines = string:split(string:trim(RawOutput), "\n", all),
    LastLine = lists:last(Lines),
    case string:prefix(LastLine, "EXIT:") of
        nomatch ->
            {1, RawOutput};
        ExitStr ->
            Code = list_to_integer(string:trim(ExitStr)),
            DiagLines = lists:droplast(Lines),
            {Code, lists:join("\n", DiagLines)}
    end.

find_project_root() ->
    Beam = code:which(typed_driver),
    case Beam of
        non_existing ->
            error(typed_driver_not_found);
        Path ->
            Abs = filename:absname(filename:dirname(Path)),
            Parts = filename:split(Abs),
            {Before, _} = lists:splitwith(
                fun(P) -> P =/= "_build" end, Parts),
            case Before of
                [] -> filename:dirname(Abs);
                _ -> filename:join(Before)
            end
    end.
