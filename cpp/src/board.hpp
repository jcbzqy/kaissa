#pragma once

#include <array>

using ZobristKey = std::uint64_t;

static uint64_t ZOBRIST_PIECE[13][64];
static uint64_t ZOBRIST_CASTLING[4];
static uint64_t ZOBRIST_EN_PASSANT[64];
static uint64_t ZOBRIST_WHITE_TO_MOVE;

enum Piece {
    EMPTY = 0,
    WP = 1,
    WN = 2,
    WB = 3,
    WR = 4,
    WQ = 5,
    WK = 6,
    BP = 7,
    BN = 8,
    BB = 9,
    BR = 10,
    BQ = 11,
    BK = 12
};

class Board {
  public:
    std::array<Piece, 64> board;

    bool whiteToMove;
    bool canWhiteCastleKingside;
    bool canWhiteCastleQueenside;
    bool canBlackCastleKingside;
    bool canBlackCastleQueenside;
    int enPassantSquare;
    int halfMoveCaptureOrPawnClock;
    int fullMoveNumber;

    ZobristKey computeZobristKey() const;
};

std::string squareToAlgebraic(int sqIndex);

std::string boardToString(const Board &boardObj);
