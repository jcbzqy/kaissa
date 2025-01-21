#include "uci.hpp"
#include "engine.hpp"

int main() {
    Engine engine;
    UCI uci(engine);
    uci.mainLoop();
    return 0;
}