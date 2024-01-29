#include "../../../src/core/webrogueMain.hpp"
#include "../../../src/outputs/sdl/SDLOutput.hpp"

#include "make_m3_runtime.h"
#include "make_wasm_c_api_runtime.h"

#include "../../embedded_resources/core_wrmod.h"
#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <JavaScriptCore/JavaScriptCore.h>

#include <string>

NSString *_Nullable writeLog2048(NSString *_Nullable dataDirectory) {
  if (dataDirectory == NULL)
    return NULL;
  NSFileManager *fileManager = [NSFileManager defaultManager];
  NSError *error = NULL;
  NSString *modDirectory = [dataDirectory stringByAppendingString:@"/mods"];

  if (![fileManager fileExistsAtPath:modDirectory]) {
    [fileManager createDirectoryAtPath:modDirectory
           withIntermediateDirectories:NO
                            attributes:NULL
                                 error:&error];
    if (error)
      return NULL;
  }
  NSString *inactiveModDirectory =
      [dataDirectory stringByAppendingString:@"/inactive_mods"];

  if (![fileManager fileExistsAtPath:inactiveModDirectory]) {
    [fileManager createDirectoryAtPath:inactiveModDirectory
           withIntermediateDirectories:NO
                            attributes:NULL
                                 error:&error];
    if (error)
      return NULL;
  }

  if ([fileManager contentsOfDirectoryAtPath:modDirectory error:&error].count ==
      0) {
    NSString *bundle =
        [[NSBundle mainBundle] pathForResource:@"log2048" ofType:@"wrmod"];
    [fileManager
        copyItemAtPath:bundle
                toPath:[modDirectory
                           stringByAppendingString:@"/log2048.wrmod"]
                 error:&error];
    if (error)
      return NULL;
  }
  if (error)
    return NULL;
  return dataDirectory;
}

NSString *_Nullable getFallbackDataDirectory() {
  NSFileManager *fileManager = [NSFileManager defaultManager];
  NSError *error = NULL;

  NSString *dataDirectory =
      [NSHomeDirectory() stringByAppendingString:@"/.webrogue_mods"];
  if (![fileManager fileExistsAtPath:dataDirectory]) {
    [fileManager createDirectoryAtPath:dataDirectory
           withIntermediateDirectories:NO
                            attributes:NULL
                                 error:&error];
    if (error)
      return NULL;
  }
  dataDirectory = writeLog2048(dataDirectory);
  return dataDirectory;
}

extern "C" NSString *_Nullable getDataDirectory() {
  NSFileManager *fileManager = [NSFileManager defaultManager];
  NSError *error = NULL;

  NSString *dataDirectory = NSSearchPathForDirectoriesInDomains(
                                NSDocumentDirectory, NSUserDomainMask, true)
                                .firstObject;
  if (dataDirectory == NULL)
    return getFallbackDataDirectory();
  dataDirectory = [dataDirectory stringByAppendingString:@"/.webrogue_mods"];
  if (![fileManager fileExistsAtPath:dataDirectory]) {
    [fileManager createDirectoryAtPath:dataDirectory
           withIntermediateDirectories:NO
                            attributes:NULL
                                 error:&error];
    if (error)
      return getFallbackDataDirectory();
  }
  dataDirectory = writeLog2048(dataDirectory);
  if (dataDirectory == NULL)
    return getFallbackDataDirectory();
  return dataDirectory;
}

extern "C" int webrogueMain() {
  NSString *modDirectory = getDataDirectory();
  if (!modDirectory)
    return 1;
  webrogue::core::Config config;
  config.addWrmodData(core_wrmod, core_wrmod_size, "core");
  config.setDataPath([modDirectory cStringUsingEncoding:NSUTF8StringEncoding]);
  config.loadsModsFromDataPath = true;
  auto output = std::make_shared<webrogue::outputs::sdl::SDLOutput>();
  output->topInset =
      [UIApplication sharedApplication].statusBarFrame.size.height;
  webrogue::core::runtime_maker_t runtime_maker =webrogue::runtimes::makeM3Runtime;
  {
    JSContext *context = [[JSContext alloc] init];
    JSValue *retValue = [context evaluateScript: @"typeof WebAssembly"];

    if([[retValue toString] isEqualToString: @"object"])
      webrogue::core::runtime_maker_t runtime_maker =webrogue::runtimes::makeWasmCApiRuntime;
  }
  return webrogue::core::webrogueMain(output, webrogue::runtimes::makeWasmCApiRuntime, &config);
}
