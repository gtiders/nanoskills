#!/bin/bash
# ---
# name: backup_files
# description: 批量备份文件到指定目录
# tags: [shell, backup, utils]
# command_template: bash {filepath} --src {source} --dst {dest}
# args:
#   source:
#     type: string
#     description: 源目录路径
#     required: true
#   dest:
#     type: string
#     description: 目标目录路径
#     required: true
# ---
echo "Backup tool"
