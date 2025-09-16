#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <wrios-Swift.h>

typedef void* DISPATCHABLE_USERDATA;
typedef void (DISPATCHABLE_FUNC)(DISPATCHABLE_USERDATA);
typedef void (DISPATCHER_FUNC)(DISPATCHABLE_FUNC, DISPATCHABLE_USERDATA);
static void dispatch_on_main_thread(DISPATCHABLE_FUNC func, DISPATCHABLE_USERDATA userdata) {
    dispatch_sync(dispatch_get_main_queue(),
    ^{
        func(userdata);
    });
}

char* webrogue_ios_rs_main(const char* path, const char* persistent_path, DISPATCHER_FUNC dispatcher);

NSString* _Nonnull webrogueObjCMain(const char* _Nonnull path, const char* _Nonnull persistent_path) {
    char* error = webrogue_ios_rs_main(path, persistent_path, dispatch_on_main_thread);
    return [[NSString alloc] initWithUTF8String: error];
}

typedef struct VkInstance_T* VkInstance;
typedef void (*PFN_vkVoidFunction)(void);
PFN_vkVoidFunction vkGetInstanceProcAddr(
    VkInstance                                  instance,
    const char*                                 pName);

UIViewController* _Nullable (^ _Nullable webrogueControllerBlock)(void) = NULL;

int webrogue_ios_main(int argc, char *argv[], UIViewController* _Nullable (^ _Nullable controllerBlock)(void))
{
    // Just to ensure MoltenVK is actually statically linked
    vkGetInstanceProcAddr(NULL, "vkCreateInstance");

    @autoreleasepool {
        webrogueControllerBlock = controllerBlock;
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([WebrogueAppDelegate class]));
    }
}
