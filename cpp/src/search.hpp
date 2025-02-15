#pragma once

#include "board.hpp"
#include "move.hpp"
#include <absl/container/flat_hash_map.h>
#include <absl/container/inlined_vector.h>

static constexpr int MAX_DEPTH = 64;
using MoveStorage = absl::InlinedVector<Move, 256>;

std::string moveToUCI(const Move &move);

void makeMove(Board &board, const Move &move);

void unmakeMove(Board &board, const Move &move, const UndoInfo &undoInfo);

MoveStorage generateLegalMoves(const Board &board);

enum NodeType {
    PV_NODE,  // Exact
    ALL_NODE, // Alpha
    CUT_NODE, // Beta
};

struct TranspositionTableEntry {
    ZobristKey key; // store key for future extensibility
    int depth;
    double value;
    NodeType nodeType;
    Move bestMove;
};

class TranspositionTable {
  public:
    bool probe(ZobristKey key, TranspositionTableEntry &entry) const;

    void store(ZobristKey key, double value, NodeType nodeType, int depth,
               const Move &bestMove);

    void clear();

  private:
    absl::flat_hash_map<ZobristKey, TranspositionTableEntry> table;
};

class Search {
  public:
    Search();
    ~Search();

    double alphaBeta(const Board &board, int depth, double alpha, double beta);

    std::optional<Move> findBestMove(
        Board &board, int depth, std::atomic<bool> &stopRequested,
        std::optional<std::chrono::milliseconds> moveTime = std::nullopt);

  private:
    double evaluate(const Board &board);
    bool isKillerMove(const Move &move, int depth) const;
    void storeKillerMove(const Move &move, int depth);

    TranspositionTable tt_;
    std::chrono::steady_clock::time_point searchStartTime_;
    std::optional<std::chrono::milliseconds> moveTimeLimit_;
    std::array<absl::InlinedVector<Move, 2>, MAX_DEPTH> killer_moves;
};