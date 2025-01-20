#pragma once

#include <string_view>

class UCI {
  private:
    bool running;

  public:
    explicit UCI();
    ~UCI() = default;

    UCI(const UCI &) = delete;
    UCI &operator=(const UCI &) = delete;
    UCI(UCI &&) = default;
    UCI &operator=(UCI &&) = default;

    void mainLoop();

    [[nodiscard]] bool processCommand(std::string_view cmd) const;

  private:
    void uciCommand() const;
    void isReadyCommand() const;
    void setOptionCommand(std::string_view name, std::string_view value) const;
    void positionCommand(std::string_view args) const;
    void goCommand(std::string_view args) const;
    void stopCommand() const;
    void quitCommand();
};
