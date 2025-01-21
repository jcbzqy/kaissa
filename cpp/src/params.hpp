#pragma once

#include <chrono>
#include <optional>
#include <string_view>
#include <vector>

struct GoParams {
    std::optional<int> searchmoves;
    std::optional<bool> ponder;
    std::optional<std::chrono::milliseconds> wtime;
    std::optional<std::chrono::milliseconds> btime;
    std::optional<std::chrono::milliseconds> winc;
    std::optional<std::chrono::milliseconds> binc;
    std::optional<int> movestogo;
    std::optional<int> depth;
    std::optional<int> nodes;
    std::optional<int> mate;
    std::optional<std::chrono::milliseconds> movetime;
    bool infinite = false;
};

struct PositionParams {
    bool is_fen;
    std::string_view position;
    std::vector<std::string_view> moves;
};

struct SearchInfo {
    std::optional<int> depth;
    std::optional<int> seldepth;
    std::optional<std::chrono::milliseconds> time;
    std::optional<uint64_t> nodes;
    std::optional<int> multipv;
    std::optional<int> score_cp;
    std::optional<int> score_mate;
    std::optional<std::string_view> pv;
};

struct OptionParams {
    std::string_view name;
    std::string_view value;
};
