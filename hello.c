#include <stdio.h>

int main()
{
    char msg[] = "Hello world\n";
    __stdio_common_vfprintf(_CRT_INTERNAL_LOCAL_PRINTF_OPTIONS, stdout, msg, NULL, NULL);
    printf("%d\n", __acrt_iob_func(1));
    return 0;
}