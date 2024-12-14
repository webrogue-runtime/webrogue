#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>

#include <string>
#include <cassert>

extern "C" void webrogue_ios_main(const char* path);

//typedef void (*onMainCallback)(void* userdata);
//
//extern "C" void webrogueRunOnMainThread(onMainCallback f, void* userdata) {
//    dispatch_sync(dispatch_get_main_queue(), ^{
//        f(userdata);
//    });
//}

extern "C" int webrogueMain(const char* path) {
    webrogue_ios_main(path);
    return 0;
}
