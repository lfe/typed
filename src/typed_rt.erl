-module(typed_rt).

-export([render_type_error/1]).

-spec render_type_error({type_error, map()}) -> string().
render_type_error({type_error, Info}) when is_map(Info) ->
    Expected = maps:get(expected, Info, unknown),
    Got = maps:get(got, Info, unknown),
    Path = maps:get(path, Info, []),
    PathStr = render_path(Path),
    lists:flatten(
      io_lib:format("type error: expected ~p~s, got ~p", [Expected, PathStr, Got]));
render_type_error(Other) ->
    lists:flatten(io_lib:format("type error: ~p", [Other])).

render_path([]) -> "";
render_path(Path) when is_list(Path) ->
    Segments = [io_lib:format(".~p", [S]) || S <- Path],
    lists:flatten([" at " | Segments]).
