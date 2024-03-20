nasm -f win64 .\example\hello-win.asm -o .\example\hello-win.obj 
@REM gcc  -m64 example/hello-win.obj  -o example/hello-win.exe
 
lld-link ./example/hello-win.obj local/libs/kernel32.lib local/libs/ucrt.lib  /entry:main /subsystem:console /out:example/hello-win.exe

@REM 00012DA0