use chess_minimax::{
    board::{self, Board},
    piece::{Piece, PieceKind},
    Pos,
    Side
};
use gdk::{
    DragAction,
    // Has to be renamed becuase gtk::prelude::* also has a DragContextExtManual
    DragContextExtManual as _DragContextTrait,
    ModifierType
};
use gdk_pixbuf::{
    prelude::*,
    Pixbuf,
    PixbufLoader
};
use gtk::{
    prelude::*,
    Align,
    Box as GtkBox,
    Button,
    CssProvider,
    DestDefaults,
    Dialog,
    DialogFlags,
    Grid,
    Image,
    Label,
    LinkButton,
    Orientation,
    ResponseType,
    StyleContext,
    TargetEntry,
    TargetFlags,
    Window,
    WindowType
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
        Arc
    },
    thread,
    time::{Duration, Instant}
};

const ICON_SIZE: i32 = 60;
const TIMEOUT: u64 = 3;
const SIDE_PLAYER: Side = Side::White;

struct Data {
    black_pawn: Pixbuf,
    black_knight: Pixbuf,
    black_bishop: Pixbuf,
    black_rook: Pixbuf,
    black_queen: Pixbuf,
    black_king: Pixbuf,
    white_pawn: Pixbuf,
    white_knight: Pixbuf,
    white_bishop: Pixbuf,
    white_rook: Pixbuf,
    white_queen: Pixbuf,
    white_king: Pixbuf
}
impl std::ops::Index<Piece> for Data {
    type Output = Pixbuf;
    fn index(&self, piece: Piece) -> &Self::Output {
        match (piece.side, piece.kind) {
            (Side::Black, PieceKind::Pawn)   => &self.black_pawn,
            (Side::Black, PieceKind::Knight) => &self.black_knight,
            (Side::Black, PieceKind::Bishop) => &self.black_bishop,
            (Side::Black, PieceKind::Rook)   => &self.black_rook,
            (Side::Black, PieceKind::Queen)  => &self.black_queen,
            (Side::Black, PieceKind::King)   => &self.black_king,
            (Side::White, PieceKind::Pawn)   => &self.white_pawn,
            (Side::White, PieceKind::Knight) => &self.white_knight,
            (Side::White, PieceKind::Bishop) => &self.white_bishop,
            (Side::White, PieceKind::Rook)   => &self.white_rook,
            (Side::White, PieceKind::Queen)  => &self.white_queen,
            (Side::White, PieceKind::King)   => &self.white_king
        }
    }
}
impl Default for Data {
    fn default() -> Self {
        Self {
            black_pawn: load(include_bytes!("res/pieces/Chess_pdt60.png")),
            black_knight: load(include_bytes!("res/pieces/Chess_ndt60.png")),
            black_bishop: load(include_bytes!("res/pieces/Chess_bdt60.png")),
            black_rook: load(include_bytes!("res/pieces/Chess_rdt60.png")),
            black_queen: load(include_bytes!("res/pieces/Chess_qdt60.png")),
            black_king: load(include_bytes!("res/pieces/Chess_kdt60.png")),
            white_pawn: load(include_bytes!("res/pieces/Chess_plt60.png")),
            white_knight: load(include_bytes!("res/pieces/Chess_nlt60.png")),
            white_bishop: load(include_bytes!("res/pieces/Chess_blt60.png")),
            white_rook: load(include_bytes!("res/pieces/Chess_rlt60.png")),
            white_queen: load(include_bytes!("res/pieces/Chess_qlt60.png")),
            white_king: load(include_bytes!("res/pieces/Chess_klt60.png"))
        }
    }
}

fn load(data: &[u8]) -> Pixbuf {
    let loader = PixbufLoader::new();
    loader.write(data).unwrap();
    loader.close().unwrap();

    loader.get_pixbuf().unwrap()
}
fn get_child(grid: &Grid, pos: Pos) -> Button {
    let Pos(x, y) = pos;
    grid.get_child_at(x as i32, y as i32).unwrap().downcast::<Button>().unwrap()
}
fn redraw(grid: &Grid, board: &Board, data: &Data) {
    for (y, row) in board.iter().enumerate() {
        for (x, piece) in row.iter().enumerate() {
            let button = get_child(grid, Pos(x as i8, y as i8));
            button.get_style_context().unwrap().remove_class("highlight");
            let image = button.get_child().unwrap().downcast::<Image>().unwrap();
            image.set_from_pixbuf(piece.map(|piece| &data[piece]));
        }
    }
}
fn main() {
    if let Err(err) = gtk::init() {
        eprintln!("failed to init gtk: {}", err);
        return;
    }

    let data = Rc::new(Data::default());

    let exit = Arc::new(AtomicBool::new(false));
    let (tx_move, rx_move) = mpsc::channel::<Board>();
    let (tx_reply, rx_reply) = mpsc::channel();
    let thread = {
        let exit = Arc::clone(&exit);
        thread::spawn(move || {
            for mut board in rx_move {
                let mut result = None;
                for depth in 1.. {
                    println!("Trying depth {}", depth);
                    if let Some(new) = board.minimax(depth, !SIDE_PLAYER, Some(&exit)) {
                        result = Some(new);
                    }
                    if exit.swap(false, Ordering::SeqCst) {
                        break;
                    }
                }
                tx_reply.send(result).unwrap();
            }
        })
    };

    let players_turn = Rc::new(Cell::new(true));
    let turn_start = Rc::new(Cell::new(None));

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Chess Minimax");
    window.set_default_size(ICON_SIZE * board::WIDTH as i32, ICON_SIZE * board::WIDTH as i32 + 200);

    let css = CssProvider::new();
    if let Err(err) = css.load_from_data(include_bytes!("res/style.css")) {
        eprintln!("failed to load css: {}", err);
        return;
    }
    StyleContext::add_provider_for_screen(
        &match window.get_screen() {
            Some(screen) => screen,
            None => {
                eprintln!("could not get default screen");
                return;
            }
        },
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
    );

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let board = Rc::new(RefCell::new(Board::new()));
    let main = GtkBox::new(Orientation::Vertical, 0);
    let grid = Grid::new();

    for (y, row) in board.borrow().iter().enumerate() {
        for (x, piece) in row.iter().enumerate() {
            let to = Pos(x as i8, y as i8);

            let icon = piece.map(|piece| &data[piece]);
            let button = Button::new();
            let image = Image::new_from_pixbuf(icon);
            image.set_size_request(ICON_SIZE, ICON_SIZE);
            button.add(&image);
            button.get_style_context().unwrap().add_class(if (y % 2 == 0) == (x % 2 == 0) {
                "white"
            } else {
                "black"
            });

            let target = &[TargetEntry::new("STRING", TargetFlags::SAME_APP, 0)];
            button.drag_source_set(ModifierType::BUTTON1_MASK, target, DragAction::DEFAULT);
            button.drag_dest_set(DestDefaults::ALL, target, DragAction::DEFAULT);
            button.connect_drag_begin(move |button, _| {
                let image = button.get_child().unwrap().downcast::<Image>().unwrap();
                if let Some(icon) = image.get_pixbuf() {
                    button.drag_source_set_icon_pixbuf(&icon);
                }
            });
            button.connect_drag_data_get(move |_button, _ctx, data, _, _| {
                data.set_text(&to.to_string());
            });

            {
                let players_turn = Rc::clone(&players_turn);
                button.connect_drag_motion(move |button, ctx, _x, _y, time| {
                    if !players_turn.get() {
                        return Inhibit(false);
                    }
                    button.get_style_context().unwrap().add_class("highlight");
                    ctx.drag_status(DragAction::MOVE, time);
                    Inhibit(true)
                });
            }
            button.connect_drag_leave(|button, _ctx, _time| {
                button.get_style_context().unwrap().remove_class("highlight");
            });

            {
                let board = Rc::clone(&board);
                let data = Rc::clone(&data);
                let grid = grid.clone();
                let players_turn = Rc::clone(&players_turn);
                let turn_start = Rc::clone(&turn_start);
                let tx_move = tx_move.clone();
                let window = window.clone();
                button.connect_drag_data_received(move |_button, ctx, _x, _y, pos, _info, time| {
                    ctx.drag_finish(true, false, time);

                    let from = match pos.get_text().and_then(|pos| pos.parse().ok()) {
                        Some(pos) => pos,
                        None => return
                    };

                    let mut board = board.borrow_mut();

                    if board.get(from).map(|p| p.side != SIDE_PLAYER).unwrap_or(true) {
                        return;
                    }

                    let mut possible = false;
                    let mut moves = board.moves_for(from);
                    while let Some(m) = moves.next(&mut board) {
                        if m == to {
                            possible = true;
                            break;
                        }
                    }

                    if !possible {
                        return;
                    }

                    let undo = board.move_(from, to);

                    if let Some(checker) = board.check(SIDE_PLAYER) {
                        board.undo(undo);
                        get_child(&grid, checker).get_style_context().unwrap().add_class("highlight");
                        return;
                    }

                    players_turn.set(false);

                    redraw(&grid, &board, &data);

                    if board.is_checkmate(!SIDE_PLAYER) {
                        let dialog = Dialog::new_with_buttons(
                            Some("Checkmate!"),
                            Some(&window),
                            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
                            &[
                                ("Ok", ResponseType::Ok.into())
                            ]
                        );
                        dialog.get_content_area().add(&Label::new("You won!"));
                        dialog.show_all();
                        dialog.run();
                        dialog.destroy();
                        return;
                    }

                    turn_start.set(Some(Instant::now()));
                    tx_move.send((*board).clone()).unwrap();
                });
            }

            grid.attach(&button, x as i32, y as i32, 1, 1);
        }
    }

    grid.set_halign(Align::Center);
    main.add(&grid);
    let attribution = LinkButton::new_with_label(
        "https://commons.wikimedia.org/wiki/Category:PNG_chess_pieces/Standard_transparent",
        "Chess pieces by Wikipedia user Cburnett - CC BY-SA 3.0"
    );
    attribution.set_vexpand(true);
    main.add(&attribution);
    window.add(&main);

    {
        let exit = Arc::clone(&exit);
        let window = window.clone();
        timeout_add_seconds(1, move || {
            if let Some(result) = rx_reply.try_recv().ok().and_then(|result| result) {
                let mut board = board.borrow_mut();
                board.move_(result.from, result.to);

                redraw(&grid, &board, &data);
                players_turn.set(true);
                turn_start.set(None);

                if board.is_checkmate(SIDE_PLAYER) {
                    let dialog = Dialog::new_with_buttons(
                        Some("Checkmate!"),
                        Some(&window),
                        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
                        &[
                            ("Ok", ResponseType::Ok.into())
                        ]
                    );
                    dialog.get_content_area().add(&Label::new("You lost!"));
                    dialog.show_all();
                    dialog.run();
                    dialog.destroy();
                }
            } else {
                if turn_start.get().map(|t| t.elapsed() >= Duration::from_secs(TIMEOUT)).unwrap_or(false) {
                    exit.store(true, Ordering::SeqCst);
                    turn_start.set(None);
                }
            }
            Continue(true)
        });
    }

    window.show_all();
    gtk::main();

    drop(tx_move); // closes the receiver
    exit.store(true, Ordering::SeqCst);
    thread.join().unwrap();
}
