#pragma once

#include "engine.hpp"
#include "params.hpp"
#include <optional>
#include <string_view>

class UCI {
  public:
    explicit UCI(Engine &engine);
    ~UCI() = default;

    UCI(const UCI &) = delete;
    UCI &operator=(const UCI &) = delete;
    UCI(UCI &&) = default;
    UCI &operator=(UCI &&) = delete;

    void mainLoop();

    [[nodiscard]] bool processCommand(std::string_view cmd) const;

  private:
    mutable bool running;
    Engine &engine;

    void uciCommand() const;
    void isReadyCommand() const;
    void debugCommand(bool on) const;
    void setOptionCommand(std::string_view name, std::string_view value) const;
    void registerCommand(std::string_view params) const;
    void positionCommand(const PositionParams &params) const;
    void goCommand(const GoParams &params) const;
    void stopCommand() const;
    void ponderHitCommand() const;
    void quitCommand() const;
    void uciNewGameCommand() const;

    void
    sendOption(std::string_view name, std::string_view type,
               std::optional<std::string_view> default_value = std::nullopt,
               std::optional<std::string_view> min = std::nullopt,
               std::optional<std::string_view> max = std::nullopt) const;
    void
    sendBestMove(std::string_view move,
                 std::optional<std::string_view> ponder = std::nullopt) const;
    void sendInfo(const SearchInfo &info) const;

    [[nodiscard]] GoParams parseGoCommand(std::string_view args) const;
    [[nodiscard]] PositionParams
    parsePositionCommand(std::string_view args) const;
    [[nodiscard]] OptionParams parseOptionCommand(std::string_view args) const;
};
