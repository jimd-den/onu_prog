#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char* as_text(long long n) __asm__("as-text");
char* joined_with(const char* a, const char* b) __asm__("joined-with");
long long onu_len(const char* s) __asm__("len");
long long onu_char_at(const char* s, long long idx) __asm__("char-at");
char* onu_init_of(const char* s) __asm__("init-of");
char* onu_char_from_code(long long code) __asm__("char-from-code");

char* as_text(long long n) {
    char* buf = malloc(32);
    sprintf(buf, "%lld", n);
    return buf;
}

char* joined_with(const char* a, const char* b) {
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char* res = malloc(len_a + len_b + 1);
    strcpy(res, a);
    strcat(res, b);
    return res;
}

long long onu_len(const char* s) {
    return (long long)strlen(s);
}

long long onu_char_at(const char* s, long long idx) {
    if (idx < 0 || idx >= strlen(s)) return 0;
    return (long long)s[idx];
}

char* onu_init_of(const char* s) {
    size_t len = strlen(s);
    if (len <= 1) return strdup("");
    char* res = malloc(len);
    strncpy(res, s, len - 1);
    res[len - 1] = '\0';
    return res;
}

char* onu_char_from_code(long long code) {
    char* res = malloc(2);
    res[0] = (char)code;
    res[1] = '\0';
    return res;
}

void broadcasts(const char* s) {
    puts(s);
}
