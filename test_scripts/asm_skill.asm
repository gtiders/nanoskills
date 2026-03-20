; ---
; name: asm_memory_optimizer
; description: Assembly 内存优化器
; tags: [asm, memory, optimizer]
; command_template: nasm -f elf64 {filepath} -o {output}
; args:
;   output:
;     type: string
;     description: 输出文件
;     required: true
; ---
section .text
global _start
_start: mov eax, 1
