#include "board.hpp"
#include <iostream>
#include <random>
#include <string>

static void initZobrist() {
    std::random_device rd;
    std::mt19937_64 gen(rd());

    for (int piece = 0; piece < 13; ++piece) {
        for (int square = 0; square < 64; ++square) {
            ZOBRIST_PIECE[piece][square] = gen();
        }
    }

    for (int i = 0; i < 4; ++i) {
        ZOBRIST_CASTLING[i] = gen();
    }

    for (int square = 0; square < 64; ++square) {
        ZOBRIST_EN_PASSANT[square] = gen();
    }

    ZOBRIST_WHITE_TO_MOVE = gen();
}

ZobristKey Board::computeZobristKey() const {
    initZobrist();

    ZobristKey key = 0ULL;

    for (int square = 0; square < 64; ++square) {
        int piece = board[square];
        if (piece != EMPTY) {
            key ^= ZOBRIST_PIECE[piece][square];
        }
    }

    if (canWhiteCastleKingside) {
        key ^= ZOBRIST_CASTLING[0];
    }
    if (canWhiteCastleQueenside) {
        key ^= ZOBRIST_CASTLING[1];
    }
    if (canBlackCastleKingside) {
        key ^= ZOBRIST_CASTLING[2];
    }
    if (canBlackCastleQueenside) {
        key ^= ZOBRIST_CASTLING[3];
    }

    if (enPassantSquare != -1) {
        key ^= ZOBRIST_EN_PASSANT[enPassantSquare];
    }

    if (whiteToMove) {
        key ^= ZOBRIST_WHITE_TO_MOVE;
    }

    return key;
}

#include <iostream>
#include <sstream>
#include <string>

inline std::string pieceToChar(Piece piece) {
    switch (piece) {
    case EMPTY:
        return ".";
    case WP:
        return "P";
    case WN:
        return "N";
    case WB:
        return "B";
    case WR:
        return "R";
    case WQ:
        return "Q";
    case WK:
        return "K";
    case BP:
        return "p";
    case BN:
        return "n";
    case BB:
        return "b";
    case BR:
        return "r";
    case BQ:
        return "q";
    case BK:
        return "k";
    default:
        return "?";
    }
}

std::string squareToAlgebraic(int sqIndex) {
    int row = sqIndex / 8;
    int col = sqIndex % 8;
    char fileChar = 'a' + static_cast<char>(col);
    char rankChar = '8' - static_cast<char>(row);
    return {fileChar, rankChar};
}

std::string boardToString(const Board &boardObj) {
    std::ostringstream oss;
    oss << "  +-----------------+\n";
    for (int rank = 7; rank >= 0; --rank) {
        oss << (rank + 1) << " | ";
        for (int file = 0; file < 8; ++file) {
            int sqIndex = (7 - rank) * 8 + file; // so rank=7 -> row 0 in array
            Piece p = boardObj.board[sqIndex];
            oss << pieceToChar(p) << " ";
        }
        oss << "|\n";
    }
    oss << "  +-----------------+\n";
    oss << "    a b c d e f g h\n\n";
    oss << (boardObj.whiteToMove ? "White" : "Black") << " to move\n";
    oss << "Castling rights: ";
    bool hasAnyCastlingRight = false;
    if (boardObj.canWhiteCastleKingside) {
        oss << "K";
        hasAnyCastlingRight = true;
    }
    if (boardObj.canWhiteCastleQueenside) {
        oss << "Q";
        hasAnyCastlingRight = true;
    }
    if (boardObj.canBlackCastleKingside) {
        oss << "k";
        hasAnyCastlingRight = true;
    }
    if (boardObj.canBlackCastleQueenside) {
        oss << "q";
        hasAnyCastlingRight = true;
    }
    if (!hasAnyCastlingRight)
        oss << "-";
    oss << "\n";
    if (boardObj.enPassantSquare >= 0 && boardObj.enPassantSquare < 64) {
        oss << "En passant target: "
            << squareToAlgebraic(boardObj.enPassantSquare) << "\n";
    } else {
        oss << "En passant target: -\n";
    }
    oss << "Halfmove clock: " << boardObj.halfMoveCaptureOrPawnClock << "\n";
    oss << "Fullmove number: " << boardObj.fullMoveNumber << "\n";
    return oss.str();
}