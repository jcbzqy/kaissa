import subprocess
import sys

class UCIEngine:
    def __init__(self, engine_path):
        self.process = subprocess.Popen(
            engine_path,
            universal_newlines=True,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            bufsize=1
        )
        self.moves = []
        self._send_and_wait("uci", "uciok")
        self._send_and_wait("isready", "readyok")
        self._send("position startpos")

    def _send(self, command):
        self.process.stdin.write(command + "\n")
        self.process.stdin.flush()

    def _send_and_wait(self, command, wait_for):
        self._send(command)
        while True:
            line = self.process.stdout.readline().strip()
            if wait_for in line:
                break

    def get_move(self, move_time=5000):
        self._send(f"go movetime {move_time}")
        while True:
            line = self.process.stdout.readline().strip()
            if line.startswith("bestmove"):
                return line.split()[1]

    def make_move(self, move):
        self.moves.append(move)
        self._send(f"position startpos moves {' '.join(self.moves)}")

    def close(self):
        self._send("quit")
        self.process.terminate()

def main():
    if len(sys.argv) != 2:
        print("Usage: python script.py <path-to-engine>")
        sys.exit(1)

    engine = UCIEngine(sys.argv[1])
    
    try:
        while True:
            move = input("Enter your move (e.g. e2e4) or 'quit' to exit: ")
            if move.lower() == 'quit':
                break

            engine.make_move(move)
            engine_move = engine.get_move()
            print(f"Engine plays: {engine_move}")
            engine.make_move(engine_move)

    except KeyboardInterrupt:
        pass
    finally:
        engine.close()

if __name__ == "__main__":
    main()