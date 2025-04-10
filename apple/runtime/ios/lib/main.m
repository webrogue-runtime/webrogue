#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <wrios-Swift.h>

void webrogue_ios_rs_main(const char* path, const char* persistent_path);

int webrogueObjCMain(const char* _Nonnull path, const char* _Nonnull persistent_path) {
    webrogue_ios_rs_main(path, persistent_path);
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
