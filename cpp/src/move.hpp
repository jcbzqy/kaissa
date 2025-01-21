#pragma once

#include "board.hpp"

struct Move {
    int from;
    int to;
    Piece promotedPiece;
    Piece capturedPiece;
    bool isEnPassant;
    bool isCastle;
};

struct UndoInfo {
    Move move;
    Piece pieceMoved;
    bool whiteToMoveBefore;
    bool canWhiteCastleKingsideBefore;
    bool canWhiteCastleQueensideBefore;
    bool canBlackCastleKingsideBefore;
    bool canBlackCastleQueensideBefore;
    int enPassantSquareBefore;
    int halfMoveCaptureOrPawnClockBefore;
    int fullMoveNumberBefore;
    uint64_t zobristKeyBefore;
};
