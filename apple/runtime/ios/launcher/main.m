#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>

void webrogue_ios_rs_main_launcher(void);

typedef struct VkInstance_T* VkInstance;
typedef void (*PFN_vkVoidFunction)(void);
PFN_vkVoidFunction vkGetInstanceProcAddr(
    VkInstance instance,
    const char* pName
);

int main(int argc, const char * argv[]) {
    // Just to ensure MoltenVK is actually statically linked
    vkGetInstanceProcAddr(NULL, "vkCreateInstance");
    webrogue_ios_rs_main_launcher();
    return 0;
}
