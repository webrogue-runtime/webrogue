#import <Foundation/Foundation.h>
#include <stdio.h>

void webrogue_macos_main(const char*, const char*);

void suicide(int sig) {
    exit(1);
}

int main(int argc, const char * argv[]) {
    signal(SIGTERM, suicide);
    // TODO use [NSApp setApplicationIconImage:] or something to change app icon at runtime
    webrogue_macos_main(argv[1], argv[2]);
    return 0;
}
