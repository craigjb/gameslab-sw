#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <unistd.h>

int main(int argc, char* argv[]) {
    if (argc < 2) {
        printf("usage: lcden {on|off}\n");
        return 1;
    }

    char enable = 1;
    if (strcmp(argv[1], "on") == 0)
        enable = 1;
    else if (strcmp(argv[1], "off") == 0)
        enable = 0;
    else {
        printf("error: %s is not a valid option.", argv[1]);
        return 1;
    }

    off_t addr = 0x43C00000;
    size_t len = sysconf(_SC_PAGE_SIZE);

    int fd = open("/dev/mem", O_RDWR | O_SYNC);
    unsigned int *mem = mmap(
        NULL, len, PROT_READ | PROT_WRITE, MAP_SHARED,
        fd, addr);
    if (mem == MAP_FAILED) {
        perror("Failed to map registers");
        return -1;
    }

    volatile unsigned int *en_reg = mem + 2;
    if (enable)
        *en_reg = 0x1;
    else
        *en_reg = 0x0;

    return 0;
}
