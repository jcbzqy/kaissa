#include "board.hpp"
#include "move.hpp"
#include "params.hpp"
#include "search.hpp"
#include "string_utils.hpp"

#include <absl/container/flat_hash_map.h>
#include <cctype>
#include <stdexcept>
#include <string>

#include <array>
#include <sstream>
#include <vector>

static void setToStartingPosition(Board &board) {
    board.board.fill(EMPTY);
    board.board[0] = BR; // a8
    board.board[1] = BN; // b8
    board.board[2] = BB; // c8
    board.board[3] = BQ; // d8
    board.board[4] = BK; // e8
    board.board[5] = BB; // f8
    board.board[6] = BN; // g8
    board.board[7] = BR; // h8
    for (int file = 0; file < 8; ++file) {
        board.board[8 + file] = BP; // a7..h7
    }
    board.board[56] = WR; // a1
    board.board[57] = WN; // b1
    board.board[58] = WB; // c1
    board.board[59] = WQ; // d1
    board.board[60] = WK; // e1
    board.board[61] = WB; // f1
    board.board[62] = WN; // g1
    board.board[63] = WR; // h1
    for (int file = 0; file < 8; ++file) {
        board.board[48 + file] = WP; // a2..h2
    }
    board.whiteToMove = true;
    board.canWhiteCastleKingside = true;
    board.canWhiteCastleQueenside = true;
    board.canBlackCastleKingside = true;
    board.canBlackCastleQueenside = true;
    board.enPassantSquare = -1;
    board.halfMoveCaptureOrPawnClock = 0;
    board.fullMoveNumber = 1;
}

static const absl::flat_hash_map<char, Piece> charToPiece = {
    {'p', BP}, {'n', BN}, {'b', BB}, {'r', BR}, {'q', BQ}, {'k', BK},
    {'P', WP}, {'N', WN}, {'B', WB}, {'R', WR}, {'Q', WQ}, {'K', WK}};

inline int algebraicToSquare(const std::string_view notation) {
    char fileChar = notation[0];
    char rankChar = notation[1];
    int col = fileChar - 'a';
    int row = '8' - rankChar;
    return row * 8 + col;
}

static Piece parsePromotionPiece(char promoChar) {
    promoChar =
        static_cast<char>(std::tolower(static_cast<unsigned char>(promoChar)));

    switch (promoChar) {
    case 'q':
        return WQ;
    case 'r':
        return WR;
    case 'n':
        return WN;
    case 'b':
        return WB;
    default:
        throw std::runtime_error(
            std::format("Invalid promotion piece character: {}", promoChar));
    }
}

static void parseFen(Board &board, std::string_view fen) {
    auto tokens = utils::split(fen);
    if (tokens.size() < 6) {
        throw std::runtime_error(
            "FEN must have at least 6 parts: [pieces] [side] [castling] "
            "[enpassant] [halfmove] [fullmove]");
    }
    board.board.fill(EMPTY);

    std::string ranksString(tokens[0]);
    std::istringstream rankStr(ranksString);

    std::vector<std::string> ranks;
    {
        std::string rankPart;
        while (std::getline(rankStr, rankPart, '/')) {
            ranks.push_back(rankPart);
        }
    }

    if (ranks.size() != 8) {
        throw std::runtime_error(
            "Invalid FEN: expected 8 ranks in piece placement");
    }
    for (int row = 0; row < 8; ++row) {
        const auto &rankData = ranks[row];
        int file = 0;
        for (char c : rankData) {
            if (std::isdigit(static_cast<unsigned char>(c))) {
                int emptyCount = c - '0';
                for (int k = 0; k < emptyCount; ++k) {
                    board.board[row * 8 + file] = EMPTY;
                    ++file;
                }
            } else {
                auto it = charToPiece.find(c);
                if (it == charToPiece.end()) {
                    throw std::runtime_error(std::format(
                        "Invalid FEN: unknown piece character: {}", c));
                } else {
                    auto piece = it->second;
                    board.board[row * 8 + file] = piece;
                    ++file;
                }
            }
        }
        if (file != 8) {
            throw std::runtime_error(
                "Invalid FEN: rank does not have 8 squares");
        }
    }

    board.whiteToMove = (tokens[1] == "w");
    {
        std::string_view castling = tokens[2];
        board.canWhiteCastleKingside =
            (castling.find('K') != std::string::npos);
        board.canWhiteCastleQueenside =
            (castling.find('Q') != std::string::npos);
        board.canBlackCastleKingside =
            (castling.find('k') != std::string::npos);
        board.canBlackCastleQueenside =
            (castling.find('q') != std::string::npos);
    }
    {
        std::string_view ep = tokens[3];
        if (ep == "-") {
            board.enPassantSquare = -1;
        } else {
            board.enPassantSquare = algebraicToSquare(ep);
        }
    }
    board.halfMoveCaptureOrPawnClock = std::stoi(std::string(tokens[4]));
    board.fullMoveNumber = std::stoi(std::string(tokens[5]));
}

static void setBoardPosition(Board &board, const PositionParams &params) {
    if (params.is_fen) {
        parseFen(board, params.position);
    } else {
        setToStartingPosition(board);
    }

    for (auto &moveStr : params.moves) {
        if (moveStr.size() < 4) {
            throw std::runtime_error(std::format(
                "Invalid move in position command: {}, move too short",
                moveStr));
        }
        auto fromSq = algebraicToSquare(moveStr.substr(0, 2));
        auto toSq = algebraicToSquare(moveStr.substr(2, 2));
        Piece promoPiece = EMPTY;
        if (moveStr.size() == 5) {
            promoPiece = parsePromotionPiece(moveStr[4]);
        }
        auto legalMoves = generateLegalMoves(board);
        bool found = false;

        for (const auto &m : legalMoves) {
            if (m.from == fromSq && m.to == toSq &&
                m.promotedPiece == promoPiece) {
                makeMove(board, m);
                found = true;
                break;
            }
        }
        if (!found) {
            throw std::runtime_error(std::string("Illegal move encountered: ") +
                                     std::string(moveStr));
        }
    }
}