#include "../../../src/core/webrogueMain.hpp"
#include "../../../src/outputs/sdl/SDLOutput.hpp"
#include <cstddef>
#include <cstdint>
#include <emscripten.h>
#include <emscripten/fetch.h>
#include <iostream>
#include <list>
#include <memory>
#include <string>
#include <vector>

void draw() {
}

extern "C" void prepareMods();
extern "C" int modsToLoadCount();
extern "C" int modToLoadNameSize();
extern "C" void modToLoadNameCopy(char *ptr);
extern "C" int modToLoadDataSize();
extern "C" void modToLoadDataCopy(uint8_t *ptr);
extern "C" void presentModSelector();

bool fetched;
webrogue::core::Config *sharedConfig;
std::string fetchedModName;

std::list<std::vector<uint8_t>> downloadedMods;

void downloadSucceeded(emscripten_fetch_t *fetch) {
    downloadedMods.push_back(
        {(const uint8_t *)(fetch->data),
         (const uint8_t *)(fetch->data + fetch->numBytes)});

    sharedConfig->addWrmodData(downloadedMods.rbegin()->data(),
                               downloadedMods.rbegin()->size(), fetchedModName);
    fetched = true;
    emscripten_fetch_close(fetch);
}

void downloadFailed(emscripten_fetch_t *fetch) {
    printf("Downloading %s failed, HTTP failure status code: %d.\n", fetch->url,
           fetch->status);
    emscripten_fetch_close(fetch); // Also free data on failure.
}

int main(int argc, char *argv[]) {
    emscripten_set_main_loop(draw, 0, 0);
    while (true) {
        auto output = std::make_shared<webrogue::outputs::sdl::SDLOutput>();
        downloadedMods.clear();
        presentModSelector();

        webrogue::core::Config config;
        sharedConfig = &config;
        config.loadsModsFromDataPath = false;
        config.setDataPath("/webrogue");

        config.onFrameEnd = []() {
            EM_ASM(FS.syncfs(function(err){}););
        };

        config.endOutputOnExit = false;

        emscripten_fetch_attr_t attr;
        emscripten_fetch_attr_init(&attr);
        strcpy(attr.requestMethod, "GET");
        attr.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY;
        attr.onsuccess = downloadSucceeded;
        attr.onerror = downloadFailed;

        fetched = false;
        fetchedModName = "core";
        emscripten_fetch(&attr, "artifacts/mods/core.wrmod");
        while (!fetched)
            emscripten_sleep(10);

        prepareMods();
        if (!modsToLoadCount()) {
            fetched = false;
            fetchedModName = "log2048";
            emscripten_fetch(&attr, "artifacts/mods/log2048.wrmod");
            while (!fetched)
                emscripten_sleep(10);
        } else {
            while (modsToLoadCount()) {
                std::vector<char> nameData;
                nameData.resize(modToLoadNameSize());
                modToLoadNameCopy(nameData.data());

                downloadedMods.push_back({});

                downloadedMods.rbegin()->resize(modToLoadDataSize());
                modToLoadDataCopy(downloadedMods.rbegin()->data());

                sharedConfig->addWrmodData(downloadedMods.rbegin()->data(),
                                           downloadedMods.rbegin()->size(),
                                           {nameData.data(), nameData.size()});
            }
        }

        webrogue::core::webrogueMain(
            output, webrogue::runtimes::makeDefaultRuntime, &config);
    }
}
