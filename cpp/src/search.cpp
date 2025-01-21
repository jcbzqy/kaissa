#include "search.hpp"

static const double pieceValues[13] = {0.0, 1.0, 3.2, 3.3, 5.0, 9.0,   1000.0,
                                       1.0, 3.2, 3.3, 5.0, 9.0, 1000.0};

static constexpr int BOARD_SIZE = 8;
static constexpr int NO_SQUARE = -1;
static const int KNIGHT_OFFSETS[8] = {-17, -15, -10, -6, 6, 10, 15, 17};
static const int KING_OFFSETS[8] = {-9, -8, -7, -1, 1, 7, 8, 9};
static const int BISHOP_DIRECTIONS[4] = {-9, -7, 7, 9};
static const int ROOK_DIRECTIONS[4] = {-8, -1, 1, 8};

inline int toIndex(int row, int col) { return row * BOARD_SIZE + col; }
inline int rowOf(int index) { return index / BOARD_SIZE; }
inline int colOf(int index) { return index % BOARD_SIZE; }
inline bool onBoard(int index) { return index >= 0 && index < 64; }
inline bool isWhitePiece(Piece p) { return p >= WP && p <= WK; }
inline bool isBlackPiece(Piece p) { return p >= BP && p <= BK; }
inline int pieceColor(Piece p) {
    if (isWhitePiece(p))
        return +1;
    if (isBlackPiece(p))
        return -1;
    return 0;
}

static bool checkDiagonalAttack(const Board &board, int square, bool white) {
    for (int d : BISHOP_DIRECTIONS) {
        int current = square;
        while (true) {
            int next = current + d;
            if (!onBoard(next)) {
                break;
            }
            if (std::abs(rowOf(next) - rowOf(current)) != 1 ||
                std::abs(colOf(next) - colOf(current)) != 1) {
                break;
            }

            Piece p = board.board[next];
            if (p != EMPTY) {
                if (white) {
                    if (p == WB || p == WQ) {
                        return true;
                    }
                } else {
                    if (p == BB || p == BQ) {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }
    return false;
}

static bool checkStraightAttack(const Board &board, int square, bool white) {
    for (int d : ROOK_DIRECTIONS) {
        int current = square;
        while (true) {
            int next = current + d;
            if (!onBoard(next)) {
                break;
            }
            if ((d == -1 || d == +1) && rowOf(next) != rowOf(current)) {
                break;
            }
            if ((d == -8 || d == +8) && colOf(next) != colOf(current)) {
                break;
            }

            Piece p = board.board[next];
            if (p != EMPTY) {
                if (white) {
                    if (p == WR || p == WQ) {
                        return true;
                    }
                } else {
                    if (p == BR || p == BQ) {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }
    return false;
}

static bool isSquareAttacked(const Board &board, int square,
                             bool attackedByWhite) {
    if (attackedByWhite) {
        int row = rowOf(square), col = colOf(square);
        if (row < 7 && col > 0) {
            int possiblePawnSq = square + 7;
            if (board.board[possiblePawnSq] == WP)
                return true;
        }
        if (row < 7 && col < 7) {
            int possiblePawnSq = square + 9;
            if (board.board[possiblePawnSq] == WP)
                return true;
        }
    } else {
        int row = rowOf(square), col = colOf(square);
        if (row > 0 && col > 0) {
            int possiblePawnSq = square - 9;
            if (board.board[possiblePawnSq] == BP)
                return true;
        }
        if (row > 0 && col < 7) {
            int possiblePawnSq = square - 7;
            if (board.board[possiblePawnSq] == BP)
                return true;
        }
    }

    for (int offset : KNIGHT_OFFSETS) {
        int knightSquare = square + offset;
        if (onBoard(knightSquare)) {
            int fromRow = rowOf(knightSquare), fromCol = colOf(knightSquare);
            int toRow = rowOf(square), toCol = colOf(square);
            if (std::abs(fromRow - toRow) <= 2 &&
                std::abs(fromCol - toCol) <= 2) {
                Piece piece = board.board[knightSquare];
                if (attackedByWhite) {
                    if (piece == WN)
                        return true;
                } else {
                    if (piece == BN)
                        return true;
                }
            }
        }
    }

    for (int offset : KING_OFFSETS) {
        int kingSquare = square + offset;
        if (onBoard(kingSquare)) {
            int fromRow = rowOf(kingSquare), fromCol = colOf(kingSquare);
            int toRow = rowOf(square), toCol = colOf(square);
            if (std::abs(fromRow - toRow) <= 1 &&
                std::abs(fromCol - toCol) <= 1) {
                Piece piece = board.board[kingSquare];
                if (attackedByWhite) {
                    if (piece == WK)
                        return true;
                } else {
                    if (piece == BK)
                        return true;
                }
            }
        }
    }

    if (attackedByWhite) {
        if (checkDiagonalAttack(board, square, true))
            return true;
        if (checkStraightAttack(board, square, true))
            return true;
    } else {
        if (checkDiagonalAttack(board, square, false))
            return true;
        if (checkStraightAttack(board, square, false))
            return true;
    }

    return false;
}

static int findKingSquare(const Board &board, bool whiteKing) {
    Piece kingPiece = whiteKing ? WK : BK;
    for (int i = 0; i < 64; i++) {
        if (board.board[i] == kingPiece)
            return i;
    }
    return NO_SQUARE;
}

bool isKingInCheck(const Board &board, bool whiteKing) {
    int kingSquare = findKingSquare(board, whiteKing);
    return isSquareAttacked(board, kingSquare, !whiteKing);
}

bool isCheckmate(const Board &board, bool whiteToMove) {
    return isKingInCheck(board, whiteToMove) &&
           generateLegalMoves(board).empty();
}

bool isStalemate(const Board &board, bool whiteToMove) {
    return !isKingInCheck(board, whiteToMove) &&
           generateLegalMoves(board).empty();
}

void makeMove(Board &board, const Move &move) {
    Piece movingPiece = board.board[move.from];
    board.board[move.from] = EMPTY;

    if (move.isEnPassant) {
        int direction = (movingPiece == WP) ? -8 : 8;
        board.board[move.to + direction] = EMPTY;
    }

    if (move.capturedPiece != EMPTY && !move.isEnPassant) {
        board.board[move.to] = EMPTY;
    }

    if (move.isCastle) {
        bool kingSide = (colOf(move.to) == 6);
        if (movingPiece == WK) {
            if (kingSide) {
                board.board[toIndex(7, 5)] = WR;
                board.board[toIndex(7, 7)] = EMPTY;
            } else {
                board.board[toIndex(7, 3)] = WR;
                board.board[toIndex(7, 0)] = EMPTY;
            }
        } else if (movingPiece == BK) {
            if (kingSide) {
                board.board[toIndex(0, 5)] = BR;
                board.board[toIndex(0, 7)] = EMPTY;
            } else {
                board.board[toIndex(0, 3)] = BR;
                board.board[toIndex(0, 0)] = EMPTY;
            }
        }
    }

    if (move.promotedPiece != EMPTY) {
        board.board[move.to] = move.promotedPiece;
    } else {
        board.board[move.to] = movingPiece;
    }

    auto disableRookCastling = [&](int sq) {
        int r = rowOf(sq), c = colOf(sq);
        if (r == 7 && c == 0)
            board.canWhiteCastleQueenside = false;
        if (r == 7 && c == 7)
            board.canWhiteCastleKingside = false;
        if (r == 0 && c == 0)
            board.canBlackCastleQueenside = false;
        if (r == 0 && c == 7)
            board.canBlackCastleKingside = false;
    };

    if (movingPiece == WK) {
        board.canWhiteCastleKingside = false;
        board.canWhiteCastleQueenside = false;
    } else if (movingPiece == BK) {
        board.canBlackCastleKingside = false;
        board.canBlackCastleQueenside = false;
    }

    if (movingPiece == WR || move.capturedPiece == WR) {
        disableRookCastling(move.from);
        if (move.capturedPiece == WR)
            disableRookCastling(move.to);
    }
    if (movingPiece == BR || move.capturedPiece == BR) {
        disableRookCastling(move.from);
        if (move.capturedPiece == BR)
            disableRookCastling(move.to);
    }

    if (movingPiece == WP && (move.to - move.from == -16)) {
        board.enPassantSquare = move.from - 8;
    } else if (movingPiece == BP && (move.to - move.from == 16)) {
        board.enPassantSquare = move.from + 8;
    } else {
        board.enPassantSquare = NO_SQUARE;
    }

    if (movingPiece == WP || movingPiece == BP || move.capturedPiece != EMPTY) {
        board.halfMoveCaptureOrPawnClock = 0;
    } else {
        ++board.halfMoveCaptureOrPawnClock;
    }

    board.whiteToMove = !board.whiteToMove;
    if (!board.whiteToMove) {
        ++board.fullMoveNumber;
    }
};

void unmakeMove(Board &board, const Move &move, const UndoInfo &undoInfo) {
    board.whiteToMove = undoInfo.whiteToMoveBefore;
    board.canWhiteCastleKingside = undoInfo.canWhiteCastleKingsideBefore;
    board.canWhiteCastleQueenside = undoInfo.canWhiteCastleQueensideBefore;
    board.canBlackCastleKingside = undoInfo.canBlackCastleKingsideBefore;
    board.canBlackCastleQueenside = undoInfo.canBlackCastleQueensideBefore;
    board.enPassantSquare = undoInfo.enPassantSquareBefore;
    board.halfMoveCaptureOrPawnClock =
        undoInfo.halfMoveCaptureOrPawnClockBefore;
    board.fullMoveNumber = undoInfo.fullMoveNumberBefore;

    Piece movingPiece = (move.promotedPiece != EMPTY) ? move.promotedPiece
                                                      : undoInfo.pieceMoved;
    board.board[move.to] = EMPTY;

    if (move.isCastle) {
        bool kingSide = (colOf(move.to) == 6 || move.to - move.from == 2);
        if (movingPiece == WK) {
            if (kingSide) {
                board.board[toIndex(7, 7)] = WR;
                board.board[toIndex(7, 5)] = EMPTY;
            } else {
                board.board[toIndex(7, 0)] = WR;
                board.board[toIndex(7, 3)] = EMPTY;
            }
        } else if (movingPiece == BK) {
            if (kingSide) {
                board.board[toIndex(0, 7)] = BR;
                board.board[toIndex(0, 5)] = EMPTY;
            } else {
                board.board[toIndex(0, 0)] = BR;
                board.board[toIndex(0, 3)] = EMPTY;
            }
        }
    }

    board.board[move.from] = undoInfo.pieceMoved;

    if (move.isEnPassant) {
        Piece captured = move.capturedPiece;
        assert(captured == WP || captured == BP);
        int direction = (captured == WP) ? -8 : 8;
        board.board[move.to + direction] = captured;
    } else if (move.capturedPiece != EMPTY) {
        board.board[move.to] = move.capturedPiece;
    }
};

inline bool isFriendlyPiece(const Board &board, Piece p) {
    return (board.whiteToMove && isWhitePiece(p)) ||
           (!board.whiteToMove && isBlackPiece(p));
}

inline bool isEnemyPiece(const Board &board, Piece p) {
    return (board.whiteToMove && isBlackPiece(p)) ||
           (!board.whiteToMove && isWhitePiece(p));
}

inline void addMove(MoveStorage &moves, int from, int to, Piece captured,
                    bool isEnPassant = false, bool isCastle = false,
                    Piece promotedPiece = EMPTY) {
    Move m{
        .from = from,
        .to = to,
        .promotedPiece = promotedPiece,
        .capturedPiece = captured,
        .isEnPassant = isEnPassant,
        .isCastle = isCastle,
    };
    moves.push_back(m);
}

void generatePawnMoves(const Board &board, int square, MoveStorage &moves) {
    Piece p = board.board[square];
    int r = rowOf(square);
    int c = colOf(square);

    bool isWhite = (p == WP);
    int forward = isWhite ? -8 : +8;
    int startRank = isWhite ? 6 : 1;
    int promotionRank = isWhite ? 0 : 7;

    int forwardOne = square + forward;
    if (onBoard(forwardOne) && board.board[forwardOne] == EMPTY) {
        if (rowOf(forwardOne) == promotionRank) {
            addMove(moves, square, forwardOne, EMPTY, false, false,
                    isWhite ? WQ : BQ);
            addMove(moves, square, forwardOne, EMPTY, false, false,
                    isWhite ? WR : BR);
            addMove(moves, square, forwardOne, EMPTY, false, false,
                    isWhite ? WB : BB);
            addMove(moves, square, forwardOne, EMPTY, false, false,
                    isWhite ? WN : BN);
        } else {
            addMove(moves, square, forwardOne, EMPTY);
        }

        if (r == startRank) {
            int forwardTwo = forwardOne + forward;
            if (forwardTwo >= 0 && forwardTwo < 64 &&
                board.board[forwardTwo] == EMPTY) {
                addMove(moves, square, forwardTwo, EMPTY);
            }
        }
    }

    for (int dc : {-1, +1}) {
        int captureCol = c + dc;
        if (captureCol < 0 || captureCol > 7)
            continue;

        int captureSquare = square + forward + dc;
        if (!onBoard(captureSquare))
            continue;

        if (isEnemyPiece(board, board.board[captureSquare])) {
            if (rowOf(captureSquare) == promotionRank) {
                addMove(moves, square, captureSquare,
                        board.board[captureSquare], false, false,
                        isWhite ? WQ : BQ);
                addMove(moves, square, captureSquare,
                        board.board[captureSquare], false, false,
                        isWhite ? WR : BR);
                addMove(moves, square, captureSquare,
                        board.board[captureSquare], false, false,
                        isWhite ? WB : BB);
                addMove(moves, square, captureSquare,
                        board.board[captureSquare], false, false,
                        isWhite ? WN : BN);
            } else {
                addMove(moves, square, captureSquare,
                        board.board[captureSquare]);
            }
        }
        if (board.enPassantSquare != NO_SQUARE &&
            captureSquare == board.enPassantSquare) {
            addMove(moves, square, captureSquare, isWhite ? BP : WP, true);
        }
    }
}

void generateKnightMoves(const Board &board, int square, MoveStorage &moves) {
    int fromRow = rowOf(square);
    int fromCol = colOf(square);

    for (int offset : KNIGHT_OFFSETS) {
        int target = square + offset;
        if (!onBoard(target)) {
            continue;
        }
        int toRow = rowOf(target);
        int toCol = colOf(target);

        int dRow = std::abs(toRow - fromRow);
        int dCol = std::abs(toCol - fromCol);
        bool isKnightMove =
            (dRow == 2 && dCol == 1) || (dRow == 1 && dCol == 2);
        if (!isKnightMove) {
            continue;
        }

        Piece targetPiece = board.board[target];
        if (targetPiece == EMPTY) {
            addMove(moves, square, target, EMPTY);
        } else if (isEnemyPiece(board, targetPiece)) {
            addMove(moves, square, target, targetPiece);
        }
    }
}

void generateBishopMoves(const Board &board, int square, MoveStorage &moves) {
    for (int d : BISHOP_DIRECTIONS) {
        int current = square;
        while (true) {
            int next = current + d;
            if (!onBoard(next)) {
                break;
            }

            int fromRow = rowOf(current), fromCol = colOf(current);
            int toRow = rowOf(next), toCol = colOf(next);
            if (std::abs(fromRow - toRow) != 1 ||
                std::abs(fromCol - toCol) != 1) {
                break;
            }

            Piece targetPiece = board.board[next];
            if (targetPiece == EMPTY) {
                addMove(moves, square, next, EMPTY);
            } else {
                if (isEnemyPiece(board, targetPiece)) {
                    addMove(moves, square, next, targetPiece);
                }
                break;
            }

            current = next;
        }
    }
}

void generateRookMoves(const Board &board, int square, MoveStorage &moves) {
    for (int d : ROOK_DIRECTIONS) {
        int current = square;
        while (true) {
            int next = current + d;
            if (!onBoard(next)) {
                break;
            }

            if (d == -1 || d == 1) {
                if (rowOf(current) != rowOf(next))
                    break;
            } else {
                if (colOf(current) != colOf(next))
                    break;
            }

            Piece targetPiece = board.board[next];
            if (targetPiece == EMPTY) {
                addMove(moves, square, next, EMPTY);
            } else {
                if (isEnemyPiece(board, targetPiece)) {
                    addMove(moves, square, next, targetPiece);
                }
                break;
            }
            current = next;
        }
    }
}

void generateQueenMoves(const Board &board, int square, MoveStorage &moves) {
    generateBishopMoves(board, square, moves);
    generateRookMoves(board, square, moves);
}

void generateKingMoves(const Board &board, int square, MoveStorage &moves) {
    for (auto &offset : KING_OFFSETS) {
        int target = square + offset;
        if (!onBoard(target) || std::abs(colOf(target) - colOf(square)) > 1)
            continue;
        Piece targetPiece = board.board[target];
        if (targetPiece == EMPTY) {
            addMove(moves, square, target, EMPTY);
        } else if (isEnemyPiece(board, targetPiece)) {
            addMove(moves, square, target, targetPiece);
        }
    }
}

void generateCastlingMoves(const Board &board, MoveStorage &moves) {
    if (board.whiteToMove) {
        int whiteKingSquare = findKingSquare(board, true);
        if (board.canWhiteCastleKingside) {
            int f1 = toIndex(7, 5);
            int g1 = toIndex(7, 6);
            if (board.board[f1] == EMPTY && board.board[g1] == EMPTY) {
                if (!isSquareAttacked(board, f1, false) &&
                    !isSquareAttacked(board, g1, false) &&
                    !isKingInCheck(board, true)) {
                    addMove(moves, whiteKingSquare, g1, EMPTY, false, true);
                }
            }
        }
        if (board.canWhiteCastleQueenside) {
            int d1 = toIndex(7, 3);
            int c1 = toIndex(7, 2);
            int b1 = toIndex(7, 1);
            if (board.board[d1] == EMPTY && board.board[c1] == EMPTY &&
                board.board[b1] == EMPTY) {
                if (!isSquareAttacked(board, d1, false) &&
                    !isSquareAttacked(board, c1, false) &&
                    !isKingInCheck(board, true)) {
                    addMove(moves, whiteKingSquare, c1, EMPTY, false, true);
                }
            }
        }
    } else {
        int blackKingSquare = findKingSquare(board, false);
        if (board.canBlackCastleKingside) {
            int f8 = toIndex(0, 5);
            int g8 = toIndex(0, 6);
            if (board.board[f8] == EMPTY && board.board[g8] == EMPTY) {
                if (!isSquareAttacked(board, f8, true) &&
                    !isSquareAttacked(board, g8, true) &&
                    !isKingInCheck(board, false)) {
                    addMove(moves, blackKingSquare, g8, EMPTY, false, true);
                }
            }
        }
        if (board.canBlackCastleQueenside) {
            int d8 = toIndex(0, 3);
            int c8 = toIndex(0, 2);
            int b8 = toIndex(0, 1);
            if (board.board[d8] == EMPTY && board.board[c8] == EMPTY &&
                board.board[b8] == EMPTY) {
                if (!isSquareAttacked(board, d8, true) &&
                    !isSquareAttacked(board, c8, true) &&
                    !isKingInCheck(board, false)) {
                    addMove(moves, blackKingSquare, c8, EMPTY, false, true);
                }
            }
        }
    }
}

// generate pseudo-legal moves, ignoring check
// for each pseudo-legal move, verify moving side king not in check
MoveStorage generateLegalMoves(const Board &board) {
    MoveStorage pseudoLegalMoves;
    MoveStorage legalMoves;

    for (int i = 0; i < 64; i++) {
        Piece p = board.board[i];
        if (p == EMPTY)
            continue;
        if ((board.whiteToMove && isWhitePiece(p)) ||
            (!board.whiteToMove && isBlackPiece(p))) {
            switch (p) {
            case WP:
            case BP:
                generatePawnMoves(board, i, pseudoLegalMoves);
                break;
            case WN:
            case BN:
                generateKnightMoves(board, i, pseudoLegalMoves);
                break;
            case WB:
            case BB:
                generateBishopMoves(board, i, pseudoLegalMoves);
                break;
            case WR:
            case BR:
                generateRookMoves(board, i, pseudoLegalMoves);
                break;
            case WQ:
            case BQ:
                generateQueenMoves(board, i, pseudoLegalMoves);
                break;
            case WK:
            case BK:
                generateKingMoves(board, i, pseudoLegalMoves);
                break;
            default:
                break;
            }
        }
    }
    generateCastlingMoves(board, pseudoLegalMoves);

    for (const Move &move : pseudoLegalMoves) {
        UndoInfo undoInfo{
            .move = move,
            .pieceMoved = board.board[move.from],
            .whiteToMoveBefore = board.whiteToMove,
            .canWhiteCastleKingsideBefore = board.canWhiteCastleKingside,
            .canWhiteCastleQueensideBefore = board.canWhiteCastleQueenside,
            .canBlackCastleKingsideBefore = board.canBlackCastleKingside,
            .canBlackCastleQueensideBefore = board.canBlackCastleQueenside,
            .enPassantSquareBefore = board.enPassantSquare,
            .halfMoveCaptureOrPawnClockBefore =
                board.halfMoveCaptureOrPawnClock,
            .fullMoveNumberBefore = board.fullMoveNumber,
            .zobristKeyBefore = board.computeZobristKey(),
        };

        makeMove(const_cast<Board &>(board), move);
        bool sideThatJustMoved = !board.whiteToMove;
        if (!isKingInCheck(board, sideThatJustMoved)) {
            legalMoves.push_back(move);
        }
        unmakeMove(const_cast<Board &>(board), move, undoInfo);
    }

    return legalMoves;
}

bool TranspositionTable::probe(ZobristKey key,
                               TranspositionTableEntry &entry) const {
    auto it = table.find(key);
    if (it == table.end()) {
        return false;
    }
    entry = it->second;
    return true;
}

void TranspositionTable::store(ZobristKey key, double value, NodeType nodeType,
                               int depth, const Move &bestMove) {
    TranspositionTableEntry entry{
        .key = key,
        .depth = depth,
        .value = value,
        .nodeType = nodeType,
        .bestMove = bestMove,
    };
    table[key] = entry;
}

void TranspositionTable::clear() { table.clear(); }

Search::Search() {}

Search::~Search() {}

double Search::evaluate(const Board &board) {
    // TODO more sophisticated than just material
    double score = 0.0;
    for (int i = 0; i < 64; i++) {
        Piece p = board.board[i];
        if (p == EMPTY)
            continue;
        double pieceValue = pieceValues[p];
        if (isWhitePiece(p)) {
            score += pieceValue;
        } else {
            score -= pieceValue;
        }
    }
    return score;
}

double Search::alphaBeta(const Board &board, int depth, double alpha,
                         double beta) {
    if (depth == 0) {
        return evaluate(board);
    }

    TranspositionTableEntry ttEntry;
    ZobristKey key = board.computeZobristKey();
    if (tt_.probe(key, ttEntry)) {
        if (ttEntry.depth >= depth) {
            switch (ttEntry.nodeType) {
            case PV_NODE:
                return ttEntry.value;
            case ALL_NODE:
                alpha = std::max(alpha, ttEntry.value);
                break;
            case CUT_NODE:
                beta = std::min(beta, ttEntry.value);
                break;
            }
            if (alpha >= beta) {
                return ttEntry.value;
            }
        }
    }

    MoveStorage moves = generateLegalMoves(board);

    if (moves.empty()) {
        if (isKingInCheck(board, board.whiteToMove)) {
            return board.whiteToMove ? -999999.0 : 999999.0;
        }
        return 0.0;
    }

    NodeType nodeType = PV_NODE;
    Move bestMove;
    double bestScore = -std::numeric_limits<double>::infinity();

    for (const auto &move : moves) {
        UndoInfo undo{
            .move = move,
            .pieceMoved = board.board[move.from],
            .whiteToMoveBefore = board.whiteToMove,
            .canWhiteCastleKingsideBefore = board.canWhiteCastleKingside,
            .canWhiteCastleQueensideBefore = board.canWhiteCastleQueenside,
            .canBlackCastleKingsideBefore = board.canBlackCastleKingside,
            .canBlackCastleQueensideBefore = board.canBlackCastleQueenside,
            .enPassantSquareBefore = board.enPassantSquare,
            .halfMoveCaptureOrPawnClockBefore =
                board.halfMoveCaptureOrPawnClock,
            .fullMoveNumberBefore = board.fullMoveNumber,
            .zobristKeyBefore = key};

        makeMove(const_cast<Board &>(board), move);
        double score = -alphaBeta(board, depth - 1, -beta, -alpha); // negamax
        unmakeMove(const_cast<Board &>(board), move, undo);

        if (score > bestScore) {
            bestScore = score;
            bestMove = move;

            if (score > alpha) {
                alpha = score;
                nodeType = PV_NODE;
            }
        }

        if (alpha >= beta) {
            nodeType = CUT_NODE;
            break;
        }
    }

    if (bestScore <= alpha) {
        nodeType = ALL_NODE;
    }
    tt_.store(key, bestScore, nodeType, depth, bestMove);
    return bestScore;
}