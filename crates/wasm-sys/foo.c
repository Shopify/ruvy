#include "./wrapper.h"
#include <unistd.h>

// XXX https://github.com/WebAssembly/wasi-libc/commit/659ff414560721b1660a19685110e484a081c3d4
pid_t getpid(void) {
    // Return an arbitrary value, greater than 1 which is special.
    return 42;
}

void asyncify_stop_rewind() {
}

void asyncify_start_unwind(int x) {
}
