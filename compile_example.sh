nasm -f elf src/hello.asm -o example/hello.o &&  ld -m elf_i386 -s example/hello.o -o example/hello