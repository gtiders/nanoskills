% ---
% name: erlang_concurrent_worker
% description: Erlang 并发工作器
% tags: [erlang, concurrent, worker]
% command_template: erl -noshell -s {filepath} start -s init stop
% args:
%   workers:
%     type: integer
%     description: 工作进程数量
%     required: true
% ---
-module(erlang_skill).
-export([start/0]).
start() -> io:format("Erlang Concurrent Worker~n").
