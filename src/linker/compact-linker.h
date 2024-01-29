#include <cstdint>
#include <functional>
#include <string>
#include <vector>

struct LinkableFile {
    LinkableFile(std::string filename, std::vector<uint8_t> data)
        : filename(filename), data(data){};

    std::string filename;
    std::vector<uint8_t> data;
};

std::vector<uint8_t> compact_link(std::vector<std::string> required_functions,
                                  std::vector<LinkableFile> files,
                                  std::function<void()> interrupt);
