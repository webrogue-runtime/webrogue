#import <Foundation/Foundation.h>
#include <stdio.h>

void webrogue_macos_main(const char*, const char*);

void suicide(int sig) {
    exit(1);
}

int main(int argc, const char * argv[]) {
    signal(SIGTERM, suicide);
    @autoreleasepool {
        NSString* exec_path = [NSString stringWithUTF8String: argv[0]];
        exec_path = [exec_path stringByDeletingLastPathComponent];
        exec_path = [exec_path stringByDeletingLastPathComponent];
        exec_path = [exec_path stringByAppendingPathComponent: @"Resources"];
        NSString* egl_path = [exec_path stringByAppendingPathComponent: @"libEGL.dylib"];
        if(![[NSFileManager defaultManager] fileExistsAtPath: egl_path]) {
            [NSException raise:@"libNotFound" format:@"libEGL.dylib not found"];
        }
        setenv("SDL_VIDEO_EGL_DRIVER", [egl_path UTF8String], 0);
        NSString* gles_path = [exec_path stringByAppendingPathComponent: @"libGLESv2.dylib"];
        if(![[NSFileManager defaultManager] fileExistsAtPath: gles_path]) {
            [NSException raise:@"libNotFound" format:@"libGLESv2.dylib not found"];
        }
        setenv("SDL_VIDEO_GL_DRIVER", [gles_path UTF8String], 0);
        NSString* wrapp_path = [exec_path stringByAppendingPathComponent: @"aot.swrapp"];
        if(![[NSFileManager defaultManager] fileExistsAtPath: wrapp_path]) {
            [NSException raise:@"libNotFound" format:@"aot.swrapp not found"];
        }
        NSURL* documentsURL = [
            [NSFileManager defaultManager]
            URLsForDirectory: NSDocumentDirectory
            inDomains: NSUserDomainMask
        ][0];
        NSString* documentsPath = [documentsURL path];
        webrogue_macos_main([wrapp_path UTF8String], [documentsPath UTF8String]);
    }
    return 0;
}
