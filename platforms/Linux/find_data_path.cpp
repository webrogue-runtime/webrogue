#include "find_data_path.hpp"
#include <dirent.h>
#include <string>

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
    abort();
}
