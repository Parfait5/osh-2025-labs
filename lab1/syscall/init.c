#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <errno.h>

#define SYS_HELLO 548  // 请确保这个编号与你的 syscall 定义一致

void test_syscall(size_t buf_len) {
    char buf[buf_len];
    long ret = syscall(SYS_HELLO, buf, buf_len);
    
    if (ret == 0) {
        printf("Syscall succeeded: %s", buf);
    } else {
        printf("Syscall failed with return value %ld (errno: %d)\n", ret, errno);
    }
}

int main() {
    printf("Testing syscall with sufficient buffer size...\n");
    test_syscall(40); // 40 足够存放完整的字符串
    
    printf("Testing syscall with insufficient buffer size...\n");
    test_syscall(10); // 10 过小，应该返回 -1
    
    return 0;
}