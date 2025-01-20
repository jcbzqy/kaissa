#include "uci.hpp"
#include "board.hpp"
#include "engine.hpp"
#include <format>
#include <iostream>

UCI::UCI() : running(false) {}

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
        const_cast<UCI *>(this)
            ->quitCommand(); // safe cast as this is a logical const break
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

void UCI::quitCommand() { running = false; }
