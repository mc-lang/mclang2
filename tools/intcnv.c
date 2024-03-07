#include<stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char** argv) {
    if (argc < 3) {
        printf("Usage: intcnv i32 134\n");
        return 1;
    }

    const char* typ = argv[1];
    __uint64_t num = atoi(argv[2]);

    if (strcmp(typ, "u8") == 0) {
        __uint8_t num2 = (__uint8_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "i8") == 0) {
        __int8_t num2 = (__int8_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "u16") == 0) {
        __uint16_t num2 = (__uint16_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "i16") == 0) {
        __int16_t num2 = (__int16_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "u32") == 0) {
        __uint32_t num2 = (__uint32_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "i32") == 0) {
        __int32_t num2 = (__int32_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "u64") == 0) {
        __uint64_t num2 = (__uint64_t)num;
        printf("%d\n", num2);
    }
    if (strcmp(typ, "i64") == 0) {
        __int64_t num2 = (__int64_t)num;
        printf("%d\n", num2);
    }

}