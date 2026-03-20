/*
---
name: c_file_reader
description: C 文件读取器
tags: [c, file, reader]
command_template: ./c_skill {filepath} --input {file}
args:
  file:
    type: string
    description: 输入文件路径
    required: true
---
*/
#include <stdio.h>
int main() {
    printf("C File Reader\n");
    return 0;
}
