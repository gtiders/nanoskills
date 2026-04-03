#!/usr/bin/env python3
# ---
# name: hello_world
# description: 一个简单的演示技能，打印问候语
# tags: [demo, test]
# command_template: python {filepath} --name {name}
# args:
#   name:
#     type: string
#     description: 要问候的名字
#     required: true
# ---
import argparse

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--name', required=True)
    args = parser.parse_args()
    print(f"Hello, {args.name}!")

if __name__ == "__main__":
    main()
