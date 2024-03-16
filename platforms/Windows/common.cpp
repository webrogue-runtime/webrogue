#include "common.hpp"
#include "shlobj.h"
#include "windows.h"
#include <iostream>
#include "shlwapi.h"

bool checkIfDirExists(std::string path) {
    DWORD dwAttrib = GetFileAttributes(path.c_str());
    return dwAttrib != INVALID_FILE_ATTRIBUTES &&
           (dwAttrib & FILE_ATTRIBUTE_DIRECTORY);
}

void initConfig(webrogue::core::Config &config) {
    char path[MAX_PATH];
    std::string stdPath;
    stdPath = "mods";
    if (checkIfDirExists(stdPath)) {
        config.setModsPath(stdPath);
    } else {
        GetModuleFileName(NULL, path, MAX_PATH);
        PathRemoveFileSpec(path);
        stdPath = path;
        stdPath += "\\mods";
        config.setModsPath(stdPath);
    }
    SHGetFolderPath(
        NULL,
        CSIDL_APPDATA | CSIDL_FLAG_CREATE,
        NULL,
        0, path
    );
    stdPath = path;
    stdPath += "\\webrogue";
    CreateDirectory(stdPath.c_str(), NULL);
    config.setDataPath(stdPath);
}
