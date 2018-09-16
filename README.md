# chess-minimax

This is a chess bot written in Rust, using the minimax algorithm with basic
alpha beta pruning.

NOTE: This branch is not finished yet. It's rewriting the original project to
have a lot cleaner and hopefully faster code. One modification is that it'll
allow playing as both sides at runtime, rather than having it be a compile time
flag.

## Terminal

If you want full control, the terminal interface is the best choice. It allows
you to use commands to change the board, even in unfair ways, and get
interesting information like technical score, and all possible moves.

![Screenshot of me being checkmated by the bot](https://i.imgur.com/SKfsQm3.png)

To get the terminal front-end, use

```
cargo run --features terminal-bin --bin terminal --release
```

You can also play against the terminal interface *on the web*
[here](https://jd91mzm2.github.io/). Press the "play chess" button, and off you
go!

## GUI (GTK+)

If your human brain has anywhere near as hard time as mine when using the
terminal front-end, don't worry! There's a basic GTK+ front-end that's there
for less control over details and more actual playing. The computer
automatically makes a move after you've made yours.

![Screenshot of me being checkmated by the bot](https://i.imgur.com/0itxWJY.png)

```
cargo run --features gtk-bin --bin gtk --release
```
