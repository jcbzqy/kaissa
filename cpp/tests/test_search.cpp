#include "board.hpp"
#include "position_utils.hpp"
#include <gtest/gtest.h>

TEST(SearchTest, StartingPositionMoveCount) {
    Board board;
    PositionParams params{.is_fen = false, .position = "startpos", .moves = {}};
    setBoardPosition(board, params);
    auto moves = generateLegalMoves(board);
    EXPECT_EQ(moves.size(), 20);
}

TEST(SearchTest, TwoCornerKingPositionMoveCount) {
    Board board;
    PositionParams params{
        .is_fen = true, .position = "k7/8/8/8/8/8/8/7K w - - 0 1", .moves = {}};
    setBoardPosition(board, params);
    auto moves = generateLegalMoves(board);
    EXPECT_EQ(moves.size(), 3);
}

TEST(SearchTest, MostPossibleMovesPositionMoveCount) {
    Board board;
    PositionParams params{
        .is_fen = true,
        .position = "3q3r/6K1/2n1b3/2q1k3/4n3/8/r7/qq1qqq1q b - - 1 59",
        .moves = {}};
    setBoardPosition(board, params);
    auto moves = generateLegalMoves(board);
    EXPECT_EQ(moves.size(), 147);
}