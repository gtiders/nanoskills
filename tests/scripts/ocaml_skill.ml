(*
---
name: ocaml_type_inferrer
description: OCaml 类型推断器
tags: [ocaml, type, inferrer]
command_template: ocaml {filepath} --expr {expression}
args:
  expression:
    type: string
    description: 表达式
    required: true
---
*)
let () = print_endline "OCaml Type Inferrer"
