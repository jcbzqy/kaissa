#pragma once
#include <string_view>
#include <vector>

namespace utils {
[[nodiscard]] inline std::vector<std::string_view> split(std::string_view str) {
    std::vector<std::string_view> tokens;
    size_t start = 0;

    while (start < str.size()) {
        while (start < str.size() && std::isspace(str[start])) {
            ++start;
        }

        if (start == str.size())
            break;

        size_t end = start + 1;
        while (end < str.size() && !std::isspace(str[end])) {
            ++end;
        }

        tokens.push_back(str.substr(start, end - start));
        start = end;
    }

    return tokens;
}
} // namespace utils
