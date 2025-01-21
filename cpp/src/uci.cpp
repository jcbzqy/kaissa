#include "uci.hpp"
#include "string_utils.hpp"
#include <format>
#include <iostream>

UCI::UCI(Engine &engine) : running(false), engine(engine) {}

void UCI::mainLoop() {
    running = true;
    std::string line;

    while (running && std::getline(std::cin, line)) {
        if (!processCommand(line)) {
            std::cerr << std::format("Unknown command: {}\n", line);
        }
    }
}

[[nodiscard]] bool UCI::processCommand(std::string_view cmd) const {
    if (cmd == "uci") {
        uciCommand();
    } else if (cmd == "isready") {
        isReadyCommand();
    } else if (cmd == "quit") {
        quitCommand();
    } else if (cmd.starts_with("debug")) {
        debugCommand(cmd.find("on") != std::string_view::npos);
    } else if (cmd.starts_with("setoption")) {
        auto params = parseOptionCommand(cmd.substr(9));
        setOptionCommand(params.name, params.value);
    } else if (cmd.starts_with("register")) {
        registerCommand(cmd.substr(9));
    } else if (cmd.starts_with("position")) {
        positionCommand(parsePositionCommand(cmd.substr(9)));
    } else if (cmd.starts_with("go")) {
        goCommand(parseGoCommand(cmd.substr(3)));
    } else if (cmd == "stop") {
        stopCommand();
    } else if (cmd == "ponderhit") {
        ponderHitCommand();
    } else {
        return false;
    }
    return true;
}

void UCI::uciCommand() const {
    std::cout << std::format("id name {}\n", "Kaissa");
    std::cout << std::format("id author {}\n", "kw");
    std::cout << "uciok\n";
}

void UCI::isReadyCommand() const { std::cout << "readyok\n"; }

void UCI::quitCommand() const { running = false; }

void UCI::sendOption(std::string_view name, std::string_view type,
                     std::optional<std::string_view> default_value,
                     std::optional<std::string_view> min,
                     std::optional<std::string_view> max) const {
    std::cout << std::format("option name {} type {}", name, type);
    if (default_value)
        std::cout << std::format(" default {}", *default_value);
    if (min)
        std::cout << std::format(" min {}", *min);
    if (max)
        std::cout << std::format(" max {}", *max);
    std::cout << '\n';
}

void UCI::sendBestMove(std::string_view move,
                       std::optional<std::string_view> ponder) const {
    std::cout << std::format("bestmove {}", move);
    if (ponder)
        std::cout << std::format(" ponder {}", *ponder);
    std::cout << '\n';
}

void UCI::sendInfo(const SearchInfo &info) const {
    std::cout << "info";
    if (info.depth)
        std::cout << std::format(" depth {}", *info.depth);
    if (info.seldepth)
        std::cout << std::format(" seldepth {}", *info.seldepth);
    if (info.time)
        std::cout << std::format(" time {}", info.time->count());
    if (info.nodes)
        std::cout << std::format(" nodes {}", *info.nodes);
    if (info.score_cp)
        std::cout << std::format(" score cp {}", *info.score_cp);
    if (info.score_mate)
        std::cout << std::format(" score mate {}", *info.score_mate);
    if (info.pv)
        std::cout << std::format(" pv {}", *info.pv);
    std::cout << '\n';
}

void UCI::registerCommand(std::string_view) const {
    std::cout << "registration ok\n";
}

void UCI::ponderHitCommand() const {
    std::cout << "ponder unsupported\n";
}

void UCI::debugCommand(bool) const {
    std::cout << "debug unsupported\n";
}

void UCI::stopCommand() const { engine.stop(); }

void UCI::setOptionCommand(std::string_view, std::string_view) const {
    std::cout << "no options available\n";
}

void UCI::positionCommand(const PositionParams &positionParams) const {
    engine.setPosition(positionParams);
}

void UCI::goCommand(const GoParams &goParams) const { engine.go(goParams); }

[[nodiscard]] PositionParams
UCI::parsePositionCommand(std::string_view args) const {
    auto tokens = utils::split(args);
    PositionParams params;

    if (tokens.empty()) {
        return params;
    }

    params.is_fen = (tokens[0] == "fen");

    if (params.is_fen) {
        size_t i = 1;
        std::string fen;
        while (i < tokens.size() && tokens[i] != "moves") {
            if (!fen.empty())
                fen += ' ';
            fen += tokens[i];
            ++i;
        }
        params.position = fen;
    } else {
        params.position = "startpos";
    }

    auto moves_it = std::find(tokens.begin(), tokens.end(), "moves");
    if (moves_it != tokens.end()) {
        params.moves.assign(moves_it + 1, tokens.end());
    }

    return params;
}

[[nodiscard]] GoParams UCI::parseGoCommand(std::string_view args) const {
    auto tokens = utils::split(args);
    GoParams params;

    for (size_t i = 0; i < tokens.size(); ++i) {
        if (tokens[i] == "infinite") {
            params.infinite = true;
        } else if (i + 1 < tokens.size()) {
            if (tokens[i] == "wtime") {
                params.wtime = std::chrono::milliseconds(
                    std::stoi(std::string(tokens[++i])));
            } else if (tokens[i] == "btime") {
                params.btime = std::chrono::milliseconds(
                    std::stoi(std::string(tokens[++i])));
            } else if (tokens[i] == "winc") {
                params.winc = std::chrono::milliseconds(
                    std::stoi(std::string(tokens[++i])));
            } else if (tokens[i] == "binc") {
                params.binc = std::chrono::milliseconds(
                    std::stoi(std::string(tokens[++i])));
            } else if (tokens[i] == "movestogo") {
                params.movestogo = std::stoi(std::string(tokens[++i]));
            } else if (tokens[i] == "depth") {
                params.depth = std::stoi(std::string(tokens[++i]));
            } else if (tokens[i] == "nodes") {
                params.nodes = std::stoi(std::string(tokens[++i]));
            } else if (tokens[i] == "mate") {
                params.mate = std::stoi(std::string(tokens[++i]));
            } else if (tokens[i] == "movetime") {
                params.movetime = std::chrono::milliseconds(
                    std::stoi(std::string(tokens[++i])));
            }
        }
    }

    return params;
}

[[nodiscard]] OptionParams
UCI::parseOptionCommand(std::string_view args) const {
    auto tokens = utils::split(args);
    OptionParams params{std::string_view{}, std::string_view{}};

    auto name_it = std::find(tokens.begin(), tokens.end(), "name");
    if (name_it != tokens.end() && (name_it + 1) != tokens.end()) {
        params.name = *(name_it + 1);
    }

    auto value_it = std::find(tokens.begin(), tokens.end(), "value");
    if (value_it != tokens.end() && (value_it + 1) != tokens.end()) {
        params.value = *(value_it + 1);
    }

    return params;
}
