#include "../../../src/core/webrogueMain.hpp"
#include "../../../src/outputs/sdl/SDLOutput.hpp"

#include "../../embedded_resources/core_wrmod.h"
#import <Foundation/Foundation.h>

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

NSString *_Nullable getDataDirectory() {
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

int main(int argc, char *argv[]) {
  NSString *modDirectory = getDataDirectory();
  if (!modDirectory)
    return 1;
  webrogue::core::Config config;
  config.addWrmodData(core_wrmod, core_wrmod_size, "core");
  config.setDataPath([modDirectory cStringUsingEncoding:NSUTF8StringEncoding]);
  config.loadsModsFromDataPath = true;
  return webrogue::core::webrogueMain(
      std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
      webrogue::runtimes::makeDefaultRuntime, &config);
}
