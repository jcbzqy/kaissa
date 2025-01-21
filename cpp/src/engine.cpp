#include "engine.hpp"
#include "position_utils.hpp"

Engine::Engine() : board_(), search_() {}
Engine::~Engine() {}

void Engine::setPosition(const PositionParams &params) {
    setBoardPosition(board_, params);
}

void Engine::go(const GoParams &params) {
    // TODO implement
    throw std::runtime_error("Not implemented");
}

void Engine::stop() {
    // TODO implement
    throw std::runtime_error("Not implemented");
}