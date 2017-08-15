# Introduction

This is version `v2` (draft) of the chess protocol.
Only breaking versions are counted.
It's something I need to make a website for my crappy-chess-minimax.
I might as well share the specification.

# Changes since last version

`INTO-QUEEN` and `CASTLING` was replaced with the superior `DIFF`.  
More detailed description of how to handle special moves.

# Specification

This protocol is designed to be easy to parse.  
Every message is a space-separated list of arguments,
where the first argument is the command.

Some messages expect replies from the server.  
These should block until they get a response (or the connection time outs).

If the client/server receives an invalid message (syntax, utf8, arguments, et.c),  
it may just shut down without further explanation.

**note**: Every command is and should be *case sensitive*.  
`INIT v0` may work, while `init v0` doesn't.

## Initialization

Every connection starts with a message from the client -
```
INIT v#
```
where # is a version number.  
The server should immediately reply with
```
ACCEPT
```
or
```
REFUSE
```
You will hear these replies mentioned a lot.  
They are the standard for allowing or denying an action.  
In this case the server should send `ACCEPT` if it supports the version
mentioned in the `INIT` message.
The server should not drop the connection but rather wait for another `INIT` command,  
unless no message has been sent within a short timeout. See [values](#values)

This should set up a chess game. The server chooses what party to play as.  
If the server decides to play black, it should send to the (still waiting!) client.  
```
BLACK
```
Otherwise, it should send
```
WHITE MOVE old new
```
where `new` and `old` are values of the old and new coordinates.  
See more about the `MOVE` command [here](#move).

## Special moves

The server should keep the client updated on all "special moves" with `DIFF`.  
The client should not need to have any knowledge of how chess is played
other than that one piece can move to another place.

**All of these are optional. You do not need to support any special moves to have a valid implementation of the protocol.**

#### Castlings

Castlings should be attempted when the client tries to `MOVE` its king to a rook.

#### Promotions

Currently the server chooses what pawns get promoted to.

## Client & Server commands.

### `MOVE`

The `MOVE` command is a command that may be used by both the server and the client.  
It takes values in the following order: X, Y, NEW X, NEW Y.  
Every value is 0-indexed.

Example:
```
MOVE H1 H3
```
This asks to move either the H1 piece two steps forward H3.

If this is received on the client, it should move the piece without any checking.  
On the server however, it might optionally do a check.  
Regardless, the server has to return either
```
ACCEPT
```
or
```
REFUSE
```
and then its move.

**note**: Position strings should *always* be UPPERCASE.

## Server-only commands

### `HIGHLIGHT`

The `HIGHLIGHT` command is used to tell the client to highlight different pieces.  
It may have any amount of arguments, as it can highlight multiple pieces.  

Example:
```
HIGHLIGHT B7 C6 D5 E4
```
*Highlight can be used to inform the client why it can't move there - for example because it would be in check.*

### `DIFF`

This command bulk-updates the board with all specified differences. Just like highlight it takes multiple arguments.  
This is used for more complicated moves, like passant, castlings and queen promotions.

The syntax is `DIFF <pos> <piece>`, where the piece is described by all lowercase `colorpiece`.

Example:
```
DIFF D1 empty C1 whiterook B1 whiteking A1 empty
```

*The above example was a castling.*

### `CHECKMATE`

The server may send this to tell the client to show some game over dialog.  

```
CHECKMATE WHITE
```
or
```
CHECKMATE BLACK
```

### `I-GIVE-UP`

The server may send this in case it gives up.  
Example scenario:  
A minimax algorithm does its best move, but is still in check.  
It would only do this if its going to lose no matter what.

```
I-GIVE-UP
```

## Client-only commands

### `SWAP`

This command simply asks the server to swap sides.  
If the server agrees (if it's supported, perhaps also check if no game has been started),  
it sends
```
ACCEPT
```
otherwise
```
REFUSE
```

# Values

These are suggestions on what magic values to use.  
You may safely disregard anything this suggests.

READ TIMEOUT INIT: 10s  
READ TIMEOUT:      2min
