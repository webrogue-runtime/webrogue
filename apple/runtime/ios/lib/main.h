#ifndef main_h
#define main_h

NSString *_Nullable getDataDirectory();
int webrogueObjCMain(const char* path);
extern UIViewController* (^webrogueControllerBlock)(void);

#endif /* main_h */
