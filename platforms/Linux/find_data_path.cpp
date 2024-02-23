#include "find_data_path.hpp"
#include <cstdio>
#include <cstdlib>
#include <dirent.h>
#include <string>
#include <sys/stat.h>
#include <unistd.h>

bool checkIfDirExists(std::string dirname) {
    DIR *dir = opendir(dirname.c_str());
    if (dir) {
        closedir(dir);
        return true;
    }
    if (ENOENT == errno) {
        return false;
    }
    return false;
}

std::string findModsPath() {
#define TRY_DIR(DIR)                                                           \
    if (checkIfDirExists(DIR))                                                 \
        return DIR;
    TRY_DIR("mods");
    TRY_DIR("./share/webrogue/mods");
    TRY_DIR("/usr/local/share/webrogue/mods");
    TRY_DIR("/usr/share/webrogue/mods");
    printf("no mods directory found. aborting");
    abort();
}

std::string findDataPath() {
    std::string dataDir = std::string(getenv("HOME")) + "/.webrogue";
    if (!checkIfDirExists(dataDir)) {
        mkdir(dataDir.c_str(), 0700);
    }
    return dataDir;
}
