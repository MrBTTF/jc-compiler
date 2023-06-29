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
    mov ebp, esp
    push word '!' | 0xa  << 8
    push 'orld'
    push 'o, w'
    push 'Hell'
    mov ecx, ebp
    sub ecx, 14


    mov edx, 14 ;message length
    ; mov ecx,msg ;message to write
    mov ebx,1   ;file descriptor (stdout)
    mov eax,4   ;system call number (sys_write)
    int 0x80    ;call kernel

; Exit via the kernel:

    mov ebx,0   ;process' exit code
    mov eax,1   ;system call number (sys_exit)
    int 0x80    ;call kernel - this interrupt won't return