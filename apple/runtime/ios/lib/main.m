#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <wrios-Swift.h>

void webrogue_ios_rs_main(const char* path);

//typedef void (*onMainCallback)(void* userdata);
//
//extern "C" void webrogueRunOnMainThread(onMainCallback f, void* userdata) {
//    dispatch_sync(dispatch_get_main_queue(), ^{
//        f(userdata);
//    });
//}

int webrogueObjCMain(const char* _Nonnull path) {
    webrogue_ios_rs_main(path);
    return 0;
}

UIViewController* _Nullable (^ _Nullable webrogueControllerBlock)(void) = NULL;

int webrogue_ios_main(int argc, char *argv[], UIViewController* _Nullable (^ _Nullable controllerBlock)(void))
{
    @autoreleasepool {
        webrogueControllerBlock = controllerBlock;
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([WebrogueAppDelegate class]));
    }
}
