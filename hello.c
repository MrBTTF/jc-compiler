#include <stdio.h>

int main()
{
    char msg[] = "Number: %d\n";
    char args[4] = {10, 0, 0, 0};
    __stdio_common_vfprintf(_CRT_INTERNAL_LOCAL_PRINTF_OPTIONS, stdout, msg, NULL, args);
    // printf("%d\n", __acrt_iob_func(1));
    return 0;
}