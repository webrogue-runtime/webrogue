#import <Foundation/Foundation.h>

char* webrogue_ios_rs_main_runner(const char* path, const char* persistent_path);

int webrogue_ios_runnerlib_main(int argc, const char * argv[]) {
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
        NSString* wrapp_path = [[NSBundle mainBundle] pathForResource:@"aot" ofType:@"swrapp"];
        
        webrogue_ios_rs_main_runner([wrapp_path UTF8String], [dataDirPath UTF8String]);
        return 0;
    }
}
