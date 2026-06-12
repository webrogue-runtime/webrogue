#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>

void webrogue_ios_rs_main_launcher(const char* persistent_path);

typedef struct VkInstance_T* VkInstance;
typedef void (*PFN_vkVoidFunction)(void);
PFN_vkVoidFunction vkGetInstanceProcAddr(
    VkInstance instance,
    const char* pName
);

int main(int argc, const char * argv[]) {
    // Just to ensure MoltenVK is actually statically linked
    vkGetInstanceProcAddr(NULL, "vkCreateInstance");

    @autoreleasepool {
        NSFileManager* fileManager = [NSFileManager defaultManager];
        
        NSString* documentDirPath = [
            [NSFileManager defaultManager]
            URLsForDirectory: NSDocumentDirectory
            inDomains: NSUserDomainMask
        ][0].relativePath;
        NSString* dataDirPath = [documentDirPath stringByAppendingPathComponent:@".webrogue"];
        dataDirPath = [documentDirPath stringByAppendingPathComponent:@"data"];
        if(![fileManager fileExistsAtPath: dataDirPath]) {
            [fileManager createDirectoryAtPath:dataDirPath withIntermediateDirectories:true attributes:NULL error:NULL];
        }
        
        webrogue_ios_rs_main_launcher([dataDirPath UTF8String]);
    }
    return 0;
}
