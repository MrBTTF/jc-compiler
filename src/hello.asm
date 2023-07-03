;to build an executable:
;       nasm -f elf -o example/hello.o src/hello.asm
;       ld -m elf_i386 -s -o example/hello example/hello.o

section .text
    global _start

section .data
msg: db  'Hello, world!',0xa ;our dear string
len: equ $ - msg         ;length of our dear string

section .text

_start:

; Write the string to stdout:
    mov rbp, rsp

    ; push rax
    mov rbx, 0xa
    shl rbx, 40
    mov rax, 'orld!'
    or rax, rbx
    push rax
    mov rax,  'Hello, w' 
    push rax
    ; pop rax

    ; push rax
    mov rax, rbp
    sub rax, 16

    mov rsi, rax ;message to write
    mov edx, 14 ;message length
    ; mov ecx,msg ;message to write
    mov edi,1   ;file descriptor (stdout)
    pop rax

    mov eax,1   ;system call number (sys_exit)
    syscall    ;call kernel


; Exit via the kernel:

    mov edi,0   ;process' exit code
    mov eax, 60   ;system call number (sys_exit)
    syscall    ;call kernel - this interrupt won't return