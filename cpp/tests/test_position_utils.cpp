#include "position_utils.hpp"
#include <gtest/gtest.h>

TEST(PositionUtilsTest, StartingPosition) {
    Board board;
    PositionParams params{.is_fen = false, .position = "startpos", .moves = {}};

    setBoardPosition(board, params);

    EXPECT_EQ(board.board[56], WR); // a1
    EXPECT_EQ(board.board[57], WN); // b1
    EXPECT_EQ(board.board[58], WB); // c1
    EXPECT_EQ(board.board[59], WQ); // d1
    EXPECT_EQ(board.board[60], WK); // e1
    EXPECT_EQ(board.board[61], WB); // f1
    EXPECT_EQ(board.board[62], WN); // g1
    EXPECT_EQ(board.board[63], WR); // h1
    for (int i = 48; i < 56; i++) {
        EXPECT_EQ(board.board[i], WP);
    }
    EXPECT_EQ(board.board[0], BR); // a8
    EXPECT_EQ(board.board[1], BN); // b8
    EXPECT_EQ(board.board[2], BB); // c8
    EXPECT_EQ(board.board[3], BQ); // d8
    EXPECT_EQ(board.board[4], BK); // e8
    EXPECT_EQ(board.board[5], BB); // f8
    EXPECT_EQ(board.board[6], BN); // g8
    EXPECT_EQ(board.board[7], BR); // h8
    for (int i = 8; i < 16; i++) {
        EXPECT_EQ(board.board[i], BP);
    }
    EXPECT_TRUE(board.whiteToMove);
    EXPECT_TRUE(board.canWhiteCastleKingside);
    EXPECT_TRUE(board.canWhiteCastleQueenside);
    EXPECT_TRUE(board.canBlackCastleKingside);
    EXPECT_TRUE(board.canBlackCastleQueenside);
    EXPECT_EQ(board.enPassantSquare, -1);
    EXPECT_EQ(board.halfMoveCaptureOrPawnClock, 0);
    EXPECT_EQ(board.fullMoveNumber, 1);
}

TEST(PositionUtilsTest, BasicFenParsing) {
    Board board;
    PositionParams params{
        .is_fen = true,
        .position = "5rk1/pp4pp/4p3/2R3Q1/3n4/2q4r/P1P2PPP/5RK1 b - - 1 23",
        .moves = {"c3g3", "g5g3", "d4e2", "g1h1", "e2g3", "f2g3", "f8f1"}};
    setBoardPosition(board, params);

    EXPECT_TRUE(board.whiteToMove);
    EXPECT_FALSE(board.canWhiteCastleKingside);
    EXPECT_FALSE(board.canWhiteCastleQueenside);
    EXPECT_FALSE(board.canBlackCastleKingside);
    EXPECT_FALSE(board.canBlackCastleQueenside);
    EXPECT_EQ(board.enPassantSquare, -1);
    EXPECT_EQ(board.halfMoveCaptureOrPawnClock, 0);
    EXPECT_EQ(board.fullMoveNumber, 26);
}