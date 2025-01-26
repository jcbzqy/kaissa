#include "engine.hpp"
#include "position_utils.hpp"
#include "search.hpp"

static int DEFAULT_DEPTH = 5;

Engine::Engine() : board_(), search_(), stopRequested_(false) {}
Engine::~Engine() { stop(); }

void Engine::setPosition(const PositionParams &params) {
    setBoardPosition(board_, params);
}

void Engine::go(const GoParams &params) {
    stop();
    stopRequested_ = false;
    searchThread_ = std::thread(&Engine::searchThreadFunction, this, params);
}

void Engine::stop() {
    stopRequested_ = true;
    if (searchThread_.joinable()) {
        searchThread_.join();
    }
}

void Engine::searchThreadFunction(GoParams params) {
    int depth = params.depth.value_or(DEFAULT_DEPTH);
    bestMove_.reset(); // clear any old
    bestMove_ =
        search_.findBestMove(board_, depth, stopRequested_, params.movetime);

    if (bestMove_.has_value()) {
        std::cout << "bestmove " << moveToUCI(bestMove_.value()) << std::endl;
    } else {
        std::cout << "bestmove 0000" << std::endl;
    }
}
