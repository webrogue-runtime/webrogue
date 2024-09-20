#import <Foundation/Foundation.h>
#include <stdio.h>

void webrogue_macos_main(void);


int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSString* exec_path = [NSString stringWithUTF8String: argv[0]];
        exec_path = [exec_path stringByDeletingLastPathComponent];
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

        NSLog(@"Hello, World!");
    }
    webrogue_macos_main();
    return 0;
}
