#pragma once

#include "board.hpp"
#include "params.hpp"
#include "search.hpp"
#include <atomic>
#include <optional>
#include <thread>
#include <chrono>

class Engine {
  public:
    Engine();
    ~Engine();

    void setPosition(const PositionParams &params);
    void go(const GoParams &params);
    void stop();

  private:
    void searchThreadFunction(GoParams params);

    Board board_;
    Search search_;

    std::thread searchThread_;
    std::atomic<bool> stopRequested_;
    std::optional<Move> bestMove_;
    std::optional<std::chrono::milliseconds> timeLimit_;
};
