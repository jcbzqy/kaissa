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

TEST(SearchTest, FindBestMoveWhiteMateInOne) {
    Board board;
    Search search;
    PositionParams params{.is_fen = true,
                          .position = "4k3/8/4K3/8/8/8/8/7R w - - 0 1",
                          .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 2, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "h1h8");
}

TEST(SearchTest, FindBestMoveBlackMateInOne) {
    Board board;
    Search search;
    PositionParams params{.is_fen = true,
                          .position = "7r/8/8/8/8/4k3/8/4K3 b - - 0 1",
                          .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 2, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "h8h1");
}

TEST(SearchTest, FindBestMoveHangingQueen) {
    Board board;
    Search search;
    PositionParams params{
        .is_fen = true,
        .position =
            "rnb1kbnr/pppp1ppp/8/4p1q1/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
        .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 2, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "f3g5");
}

TEST(SearchTest, FindBestMoveKnightFork) {
    Board board;
    Search search;
    PositionParams params{.is_fen = true,
                          .position = "8/4k3/7q/8/8/4N3/4K3/4R3 w - - 0 1",
                          .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 4, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "e3f5");
}

TEST(SearchTest, FindBestMoveQueenSacSmotheredMate) {
    Board board;
    Search search;
    PositionParams params{
        .is_fen = true,
        .position = "r6k/1p1b1Qbp/1n2B1pN/p7/Pq6/8/1P4PP/R6K w - - 1 27",
        .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 4, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "f7g8");
}

TEST(SearchTest, FindBestMoveOpeningHangingBishop) {
    Board board;
    Search search;
    PositionParams params{
        .is_fen = true,
        .position = "rnb1kbnr/ppqppppp/2p5/1B6/3PP3/2P5/PP3PPP/RNBQK1NR b KQkq - 0 1",
        .moves = {}};
    setBoardPosition(board, params);
    std::atomic<bool> stop(false);
    auto bestMove = search.findBestMove(board, 3, stop);
    ASSERT_TRUE(bestMove.has_value());
    EXPECT_EQ(moveToUCI(bestMove.value()), "c6b5");
}