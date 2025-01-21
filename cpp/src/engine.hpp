#pragma once

#include "board.hpp"
#include "params.hpp"
#include "search.hpp"

class Engine {
  public:
    Engine();
    ~Engine();

    void setPosition(const PositionParams &params);
    void go(const GoParams &params);
    void stop();

  private:
    Board board_;
    Search search_;
};
