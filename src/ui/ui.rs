use std::path::Path;
use std::time::Instant;

use super::config::{Clock, GameMode, PromotionChoice, Promotions, UIConfig};
use super::engine::{EngineStatus, UIengine};
use super::settings::{SettingsMessage, SettingsTab};
use super::styling::button::CustomButtonStyle;
use super::styling::container::container_appearance;
use super::tournament::{self, Tournament};
use crate::board::defs::{Pieces, Squares, SQUARE_NAME};
use crate::board::Board;
use crate::defs::{Sides, Square};
use crate::movegen::defs::{Move, MoveList, MoveType, Shift};
use crate::movegen::MoveGenerator;
use iced::alignment::{Horizontal, Vertical};
use iced::theme::ProgressBar;
use iced::widget::container::StyleSheet;
use iced::widget::{
    column, image, progress_bar, responsive, row, Button, Column, Container, Image, Radio, Row,
    Svg, Text,
};
use iced::{
    executor, Alignment, Application, Color, Command, Element, Length, Sandbox, Settings, Size,
    Subscription, Theme,
};
use tokio::sync::mpsc::Sender;

pub struct Editor {
    board: Board,
    engine1: UIengine,
    engine2: UIengine,
    engine1_status: EngineStatus,
    engine2_status: EngineStatus,
    movegen: MoveGenerator,
    settings: SettingsTab,
    from_square: Option<Square>,
    engine1_sender: Option<Sender<String>>,
    engine2_sender: Option<Sender<String>>,
    highlighted_squares: Vec<Square>,
    promotion: Promotions,
    eval: f32,
    tournament: Option<Tournament>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Settings(SettingsMessage),
    ChangeSettings(Option<UIConfig>),
    SelectSquare(Option<Square>),
    EngineMove(Option<usize>, Option<usize>, Option<usize>),
    EventOccurred(iced::Event),
    StartEngine,
    EngineReady(Sender<String>),
    EngineStopped(bool),
    UndoMove,
    UndoMoveVirtual,
    NextMoveVirtual,
    ResetBoardEngine,
    PrintLegalMoves,
    ChangeStartPos,
    SelectSideToMove(usize),
    PromotionSelected(PromotionChoice),
    Tick,
    UpdateTime,
    UpdateEval(f32),
    RawMove(Vec<String>),
    StartTournament,
    NextGame,
    GameFinished(String),
    LogResult(String),
}

pub fn run() -> iced::Result {
    Editor::run(Settings {
        window: iced::window::Settings {
            size: (1130, 1080),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                board: Board::build(),
                engine1: UIengine::new(
                    "/Users/wouter/personal/rust/chess-engine/target/release/chess-engine"
                        .to_string(),
                    3,
                ),
                engine2: UIengine::new(
                    "/Users/wouter/personal/rust/chess-engine/target/debug/chess-engine"
                        .to_string(),
                    3,
                ),
                engine1_status: EngineStatus::TurnedOff,
                engine2_status: EngineStatus::TurnedOff,
                movegen: MoveGenerator::new(),
                settings: SettingsTab::new(),
                from_square: None,
                engine1_sender: None,
                engine2_sender: None,
                highlighted_squares: vec![],
                promotion: Promotions::default(),
                eval: 50.0,
                tournament: None,
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        String::from("Chess app")
    }

    fn update(&mut self, message: self::Message) -> Command<Message> {
        match (self.from_square, message) {
            (None, Message::SelectSquare(pos)) => {
                let side = self.board.side_to_move();
                let color = self.board.color_on(pos);

                println!("{:?}", pos);

                // Reset highlighted squares
                self.highlighted_squares.clear();

                // if user clicked on another square with a piece his own side reset
                if color == side {
                    self.from_square = pos;
                }

                let mut legal_moves = MoveList::new();
                let piece = &self.board.piece_on(pos);
                // Get all psuedo legal moves for the position. and the selected piece on the
                // square
                if let Some(piece) = piece {
                    match *piece {
                        Pieces::PAWN => {
                            self.movegen
                                .pawns(&self.board, &mut legal_moves, MoveType::All)
                        }
                        Pieces::KING => self.movegen.piece(
                            &self.board,
                            Pieces::KING,
                            &mut legal_moves,
                            MoveType::All,
                        ),
                        Pieces::QUEEN => self.movegen.piece(
                            &self.board,
                            Pieces::QUEEN,
                            &mut legal_moves,
                            MoveType::All,
                        ),
                        Pieces::ROOK => self.movegen.piece(
                            &self.board,
                            Pieces::ROOK,
                            &mut legal_moves,
                            MoveType::All,
                        ),
                        Pieces::BISHOP => self.movegen.piece(
                            &self.board,
                            Pieces::BISHOP,
                            &mut legal_moves,
                            MoveType::All,
                        ),
                        Pieces::KNIGHT => self.movegen.piece(
                            &self.board,
                            Pieces::KNIGHT,
                            &mut legal_moves,
                            MoveType::All,
                        ),
                        _ => (),
                    }
                }

                for move_data in legal_moves
                    .moves
                    .iter()
                    .filter(|move_data| move_data.from() == pos.unwrap())
                {
                    // Get all legal moves for the square and change it's color
                    self.highlighted_squares.push(move_data.to());
                }
                Command::none()
            }
            (Some(from), Message::SelectSquare(to)) if from != to.unwrap() => {
                // Reset highlighted squares
                self.highlighted_squares.clear();

                let side = self.board.side_to_move();
                let color = self.board.color_on(to);

                if color == side {
                    // If user clicked on another square with a piece of his own side, update from_square and legal moves
                    self.from_square = to;

                    let mut legal_moves = MoveList::new();
                    let piece = self.board.piece_on(to);

                    // Get all pseudo-legal moves for the position and the selected piece on the square
                    if let Some(piece) = piece {
                        match piece {
                            Pieces::PAWN => {
                                self.movegen
                                    .pawns(&self.board, &mut legal_moves, MoveType::All)
                            }
                            Pieces::KING => self.movegen.piece(
                                &self.board,
                                Pieces::KING,
                                &mut legal_moves,
                                MoveType::All,
                            ),
                            Pieces::QUEEN => self.movegen.piece(
                                &self.board,
                                Pieces::QUEEN,
                                &mut legal_moves,
                                MoveType::All,
                            ),
                            Pieces::ROOK => self.movegen.piece(
                                &self.board,
                                Pieces::ROOK,
                                &mut legal_moves,
                                MoveType::All,
                            ),
                            Pieces::BISHOP => self.movegen.piece(
                                &self.board,
                                Pieces::BISHOP,
                                &mut legal_moves,
                                MoveType::All,
                            ),
                            Pieces::KNIGHT => self.movegen.piece(
                                &self.board,
                                Pieces::KNIGHT,
                                &mut legal_moves,
                                MoveType::All,
                            ),
                            _ => (),
                        }
                    }

                    for move_data in legal_moves
                        .moves
                        .iter()
                        .filter(|move_data| move_data.from() == to.unwrap())
                    {
                        // Highlight squares for all legal moves from the selected square
                        self.highlighted_squares.push(move_data.to());
                    }

                    return Command::none();
                }

                let mut legal_moves = MoveList::new();
                // Get all pseudo-legal moves for the position.
                self.movegen
                    .generate_moves(&self.board, &mut legal_moves, MoveType::All);

                if self.board.piece_on(Some(from)).unwrap() == Pieces::PAWN {
                    // Check if the pawn is moving to the promotion rank
                    let promotion_rank = Board::promotion_rank(side);
                    if to.unwrap() / 8 == promotion_rank {
                        // Show the promotion prompt
                        self.promotion.show_promotion_prompt = true;
                        self.promotion.promotion_square = to;
                        return Command::none();
                    }
                }

                self.from_square = None;

                // Get data needed for converting algebraic move to Move data
                let side = self.board.side_to_move() == Sides::WHITE;
                let move_data = self.board.generate_move_data(&from, &to, side, None);

                // Check if move is legal
                if legal_moves.moves.iter().any(|x| x.data == move_data) {
                    self.board.make_move(Move::new(move_data), &self.movegen);
                } else {
                    println!(
                        "{:?}\n",
                        legal_moves
                            .moves
                            .iter()
                            .filter(|x| x.data > 0)
                            //.map(|f| SQUARE_NAME[f.from()].to_owned() + SQUARE_NAME[f.to()])
                            .collect::<Vec<_>>()
                    );
                    println!("illegal move");
                }

                // Only if Engine is playing against humans and only if it is not the player's turn
                if self.settings.game_mode == GameMode::PlayerEngine {
                    if !(self.settings.player_side as usize == self.board.side_to_move()) {
                        if let Some(sender) = &self.engine1_sender {
                            if let Err(e) = sender.blocking_send(self.board.create_fen()) {
                                eprintln!("Lost connection with the engine: {}", e);
                            }
                        }
                    }
                }

                if self.settings.game_mode == GameMode::EngineEngine {
                    if let Some(sender) = &self.engine1_sender {
                        if let Err(e) = sender.blocking_send(self.board.create_fen()) {
                            eprintln!("Lost connection with the engine: {}", e);
                        }
                    }
                    if let Some(sender) = &self.engine2_sender {
                        if let Err(e) = sender.blocking_send(self.board.create_fen()) {
                            eprintln!("Lost connection with the engine: {}", e);
                        }
                    }
                }

                Command::none()
            }
            (_, Message::EngineMove(from, to, promotion)) => {
                // Let engine make move

                Command::none()
            }
            (_, Message::StartTournament) => {
                self.tournament = Some(Tournament::new(1, "./tournament_log.txt").unwrap());
                Command::perform(async { Message::NextGame }, |msg| msg)
            }
            (_, Message::NextGame) => {
                // Check if the tournament is over
                if let Some(tournament) = &self.tournament {
                    if tournament.tournament_over() {
                        println!("Tournament over!");
                        return Command::none();
                    }
                }
                // Start the next game
                // Reset board and start engines
                self.board = Board::build();
                self.engine1_status = EngineStatus::TurnedOn;
                self.engine2_status = EngineStatus::TurnedOn;
                if let Some(sender) = &self.engine1_sender {
                    sender.blocking_send(self.board.create_fen()).unwrap();
                }
                if let Some(sender) = &self.engine2_sender {
                    sender.blocking_send(self.board.create_fen()).unwrap();
                }
                Command::none()
            }
            (_, Message::GameFinished(result)) => {
                if let Some(tournament) = &mut self.tournament {
                    tournament.log_result(&result).unwrap();
                }
                Command::perform(async { Message::NextGame }, |msg| msg)
            }
            (_, Message::StartEngine) => {
                if self.settings.game_mode == GameMode::EngineEngine {
                    match self.engine2_status {
                        EngineStatus::TurnedOff => {
                            if self.engine2_status == EngineStatus::TurnedOff {
                                if Path::new(&self.engine2.engine_path).exists() {
                                    self.engine1.position = self.board.create_fen();
                                    self.engine1_status = EngineStatus::TurnedOn;
                                } else {
                                    println!("Invalid engine 1 path");
                                }
                            } else {
                                // Turn off engines if not in EngineEngine mode
                                self.engine2_status = EngineStatus::TurnedOff;
                            }
                        }
                        _ => {
                            if let Some(sender) = &self.engine1_sender {
                                sender
                                    .blocking_send(String::from("STOP"))
                                    .expect("Error quiting engine");
                                self.engine1_sender = None;
                            }
                        }
                    }
                }

                match self.engine1_status {
                    EngineStatus::TurnedOff => {
                        if self.engine1_status == EngineStatus::TurnedOff {
                            if Path::new(&self.engine1.engine_path).exists() {
                                self.engine1.position = self.board.create_fen();
                                self.engine1_status = EngineStatus::TurnedOn;
                            } else {
                                println!("Invalid engine 1 path");
                            }
                        } else {
                            // Turn off engines if not in EngineEngine mode
                            self.engine1_status = EngineStatus::TurnedOff;
                        }
                    }
                    _ => {
                        if let Some(sender) = &self.engine1_sender {
                            sender
                                .blocking_send(String::from("STOP"))
                                .expect("Error quiting engine");
                            self.engine1_sender = None;
                        }
                    }
                }
                Command::none()
            }
            (_, Message::EngineReady(message)) => {
                if self.engine1_status == EngineStatus::TurnedOn && self.engine1_sender.is_none() {
                    self.engine1_sender = Some(message);
                } else if self.engine2_status == EngineStatus::TurnedOn
                    && self.engine2_sender.is_none()
                {
                    self.engine2_sender = Some(message);
                }
                if self.settings.game_mode == GameMode::EngineEngine {
                    if let Some(engine1_sender) = &self.engine1_sender {
                        if let Err(e) = engine1_sender.blocking_send(self.board.create_fen()) {
                            eprintln!("Lost connection with engine 1: {}", e);
                        }
                    }
                }
                Command::none()
            }
            (_, Message::EventOccurred(_event)) => {
                //
                Command::none()
            }
            (_, Message::PrintLegalMoves) => {
                let mut legal_moves = MoveList::new();
                // Get all pseudo-legal moves for the position.
                self.movegen
                    .generate_moves(&self.board, &mut legal_moves, MoveType::All);

                for mov in legal_moves.moves.iter() {
                    if mov.data > 0 {
                        println!("{}{}", SQUARE_NAME[mov.from()], SQUARE_NAME[mov.to()]);
                        // println!(
                        //     " - {}",
                        //     self.movegen.square_attacked(
                        //         &self.board,
                        //         self.board.side_to_not_move(),
                        //         Squares::G8
                        //     )
                        // )
                    }
                }

                Command::none()
            }
            (_, Message::Settings(message)) => self.settings.update(message),
            (_, Message::ChangeSettings(message)) => {
                if let Some(settings) = message {
                    self.settings.flip_board = settings.flip_board;
                    self.settings.show_coords = settings.show_coordinates;
                    self.settings.search_depth = settings.search_depth;
                    self.settings.game_mode = settings.game_mode;
                }
                Command::none()
            }
            (_, Message::SelectSideToMove(_message)) => {
                self.board.swap_side();
                Command::none()
            }
            (_, Message::UndoMoveVirtual) => {
                if self.board.history.len() > 0 {
                    self.board.unmake();
                }
                Command::none()
            }
            (_, Message::NextMoveVirtual) => {
                //
                Command::none()
            }
            (_, Message::ChangeStartPos) => {
                // update board
                let _ = self
                    .board
                    .read_fen(Some("3r4/3r4/3k4/8/8/8/8/3K4 w - - 0 1"));

                // update engine
                self.engine1.position = "3r4/3r4/3k4/8/8/8/8/3K4 w - - 0 1".to_string();
                self.engine2.position = "3r4/3r4/3k4/8/8/8/8/3K4 w - - 0 1".to_string();
                Command::none()
            }
            (_, Message::PromotionSelected(choice)) => {
                if let Some(to) = self.promotion.promotion_square {
                    let from = self.from_square.unwrap();
                    let side = self.board.side_to_move() == Sides::WHITE;
                    let move_data = self.board.generate_move_data(
                        &from,
                        &Some(to),
                        side,
                        Some(choice as usize + 1), // + 1 bc queen starts at 0,
                    );

                    self.board.make_move(Move::new(move_data), &self.movegen);
                    self.promotion.show_promotion_prompt = false; // Hide the promotion prompt
                    self.promotion.promotion_square = None; // Reset the promotion square
                                                            //
                    if self.settings.game_mode == GameMode::PlayerEngine {
                        if !(self.settings.player_side as usize == self.board.side_to_move()) {
                            if let Some(sender) = &self.engine1_sender {
                                if let Err(e) = sender.blocking_send(self.board.create_fen()) {
                                    eprintln!("Lost connection with the engine: {}", e);
                                }
                            }
                        }
                    }
                }
                Command::none()
            }
            (_, Message::UpdateTime) => Command::none(),
            (_, Message::Tick) => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.engine1.clock.last_tick).as_millis();

                if self.board.side_to_move() == Sides::WHITE {
                    self.engine1.clock.wtime = self.engine1.clock.wtime.saturating_sub(elapsed);
                } else {
                    self.engine1.clock.btime = self.engine1.clock.btime.saturating_sub(elapsed);
                }

                self.engine1.clock.last_tick = now;

                Command::none()
            }
            (_, Message::UpdateEval(eval)) => {
                println!("{:?}", eval);
                self.eval = eval;
                Command::none()
            }
            (_, Message::RawMove(themove)) => {
                //println!("{:?}", themove);
                Command::none()
            }
            (_, _) => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.engine1_status {
            EngineStatus::TurnedOff => iced::subscription::events().map(Message::EventOccurred),
            _ => Subscription::batch(vec![
                self.engine1.clone().run_engine(),
                iced::subscription::events().map(Message::EventOccurred),
                iced::time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick),
            ]),
        }
    }

    fn view(&self) -> Element<Message, iced::Renderer<Theme>> {
        let resp = responsive(move |size| {
            main_view(
                &self.board,
                self.settings.flip_board,
                self.settings.show_coords,
                self.settings.search_depth,
                self.settings.game_mode,
                self.settings.view(),
                self.engine1_status != EngineStatus::TurnedOff,
                size,
                &self.highlighted_squares,
                &self.promotion,
                &self.engine1,
                self.eval,
            )
        });

        Container::new(resp).padding(1).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main_view<'a>(
    board: &Board,
    flip_board: bool,
    show_coordinates: bool,
    _search: u32,
    game_mode: GameMode,
    settings_tab: Element<'a, Message, iced::Renderer<Theme>>,
    engine_started: bool,
    _size: Size,
    highlighted_squares: &Vec<Square>,
    promotion: &Promotions,
    engine: &UIengine,
    eval: f32,
) -> Element<'a, Message, iced::Renderer<Theme>> {
    let mut board_col = Column::new().spacing(0).align_items(Alignment::Center);
    let mut board_row = Row::new().spacing(0).align_items(Alignment::Center);
    let mut promotion_counter = 0;
    let promotion_options = vec![
        ("/wQ.svg", PromotionChoice::Queen),
        ("/wN.svg", PromotionChoice::Knight),
        ("/wR.svg", PromotionChoice::Rook),
        ("/wB.svg", PromotionChoice::Bishop),
    ];

    let timer_row = column![
        Button::new(Text::new(format!(
            "Black: {}:{:02}",
            engine.clock.btime / 60000,
            (engine.clock.btime % 60000) / 1000
        ))),
        // .style(
        //     CustomButtonStyle::new()
        //         .background_color(Color::from_rgb(1.0, 1.0, 1.0))
        //         .as_custom()
        // ),
        Button::new(Text::new(format!(
            "White: {}:{:02}",
            engine.clock.wtime / 60000,
            (engine.clock.wtime % 60000) / 1000
        ))) // .style(
            //     CustomButtonStyle::new()
            //         .background_color(Color::from_rgb(1.0, 0.0, 0.0)) // Highlight color
            //         .hovered()
            //         .background_color(Color::from_rgb(0.4, 0.4, 0.8))
            //         .pressed()
            //         .background_color(Color::from_rgb(0.3, 0.3, 0.7))
            //         .as_custom()
            // ),
    ]
    .padding(30)
    .spacing(90 * 8);

    let ranks;
    let files;
    ranks = (1..=8).collect::<Vec<i32>>();
    files = (1..=8).rev().collect::<Vec<i32>>();
    let board_height = 100;
    let row = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

    for rank in ranks {
        for file in &files {
            let pos = board.get_square((rank as usize, *file as usize));

            let (piece, color) = (board.piece_on(pos), board.color_on(pos));

            let mut text = "";
            let light_square = (rank + file) % 2 != 0;

            // let selected = from_square == Some(pos);
            let square_style = if highlighted_squares.contains(&pos.unwrap()) {
                CustomButtonStyle::new()
                    .background_color(Color::from_rgb(1.0, 0.0, 0.0)) // Highlight color
                    .hovered()
                    .background_color(Color::from_rgb(0.4, 0.4, 0.8))
                    .pressed()
                    .background_color(Color::from_rgb(0.3, 0.3, 0.7))
            } else if light_square {
                CustomButtonStyle::new()
                    .background_color(Color::from_rgb(0.91, 0.741, 0.529))
                    .hovered()
                    .background_color(Color::from_rgb(0.91, 0.741, 0.529))
                    .pressed()
                    .background_color(Color::from_rgb(0.803, 0.82, 0.415))
            } else {
                CustomButtonStyle::new()
                    .background_color(Color::from_rgb(0.639, 0.502, 0.329))
                    .hovered()
                    .background_color(Color::from_rgb(0.639, 0.502, 0.329))
                    .pressed()
                    .background_color(Color::from_rgb(0.666, 0.635, 0.22))
            };

            let button;

            // Show promotion pieces
            if promotion.show_promotion_prompt
                && promotion_counter < 4
                && (pos
                    == Some(promotion.promotion_square.unwrap() as usize - promotion_counter * 8))
            {
                let (svg_path, piece) = &promotion_options[promotion_counter];

                button = Button::new(Svg::from_path(format!(
                    "{}/pieces{}",
                    env!("CARGO_MANIFEST_DIR"),
                    svg_path
                )))
                .width(board_height)
                .height(board_height)
                .on_press(Message::PromotionSelected(piece.clone()))
                .style(square_style.as_custom());

                promotion_counter += 1;
            } else {
                // Set pieces on boad
                button = if let Some(piece) = piece {
                    if color == Sides::WHITE {
                        text = match piece {
                            Pieces::PAWN => "/wP.svg",
                            Pieces::ROOK => "/wR.svg",
                            Pieces::KNIGHT => "/wN.svg",
                            Pieces::BISHOP => "/wB.svg",
                            Pieces::QUEEN => "/wQ.svg",
                            Pieces::KING => "/wK.svg",
                            _ => "",
                        };
                    } else {
                        text = match piece {
                            Pieces::PAWN => "/bP.svg",
                            Pieces::ROOK => "/bR.svg",
                            Pieces::KNIGHT => "/bN.svg",
                            Pieces::BISHOP => "/bB.svg",
                            Pieces::QUEEN => "/bQ.svg",
                            Pieces::KING => "/bK.svg",
                            _ => "",
                        };
                    }

                    Button::new(Svg::from_path(format!(
                        "{}/pieces{}",
                        env!("CARGO_MANIFEST_DIR"),
                        text
                    )))
                    .width(board_height)
                    .height(board_height)
                    .on_press(Message::SelectSquare(pos))
                    .style(square_style.as_custom())
                } else {
                    Button::new(Text::new(""))
                        .width(board_height)
                        .height(board_height)
                        .on_press(Message::SelectSquare(pos))
                        .style(square_style.as_custom())
                };
            }

            board_col = board_col.push(button);
        }

        if show_coordinates {
            board_col = board_col.push(
                Container::new(Text::new((row[rank as usize - 1]).to_string()).size(15))
                    .align_y(iced::alignment::Vertical::Top)
                    .align_x(iced::alignment::Horizontal::Left)
                    .padding(5)
                    .width(board_height),
            );
        }

        board_row = board_row.push(board_col);
        board_col = Column::new().spacing(0).align_items(Alignment::Center);
    }

    if show_coordinates {
        if !flip_board {
            board_row = board_row.push(
                column![
                    Text::new("8").size(15).height(board_height),
                    Text::new("7").size(15).height(board_height),
                    Text::new("6").size(15).height(board_height),
                    Text::new("5").size(15).height(board_height),
                    Text::new("4").size(15).height(board_height),
                    Text::new("3").size(15).height(board_height),
                    Text::new("2").size(15).height(board_height),
                    Text::new("1").size(15).height(board_height),
                ]
                .padding(5),
            );
        } else {
            board_row = board_row.push(column![
                Text::new("1").size(15).height(board_height),
                Text::new("2").size(15).height(board_height),
                Text::new("3").size(15).height(board_height),
                Text::new("4").size(15).height(board_height),
                Text::new("5").size(15).height(board_height),
                Text::new("6").size(15).height(board_height),
                Text::new("7").size(15).height(board_height),
                Text::new("8").size(15).height(board_height),
            ]);
        }
    }

    let mut side_to_play = row![];

    if board.side_to_move() == Sides::WHITE {
        side_to_play = side_to_play.push(Text::new("White to move"));
    } else {
        side_to_play = side_to_play.push(Text::new("Black to move"));
    }

    let game_mode_row = row![
        Text::new("Play as"),
        Radio::new(
            "White",
            Sides::WHITE,
            Some(board.side_to_move()),
            Message::SelectSideToMove
        ),
        Radio::new(
            "Black",
            Sides::BLACK,
            Some(board.side_to_move()),
            Message::SelectSideToMove
        )
    ]
    .spacing(10)
    .padding(10)
    .align_items(Alignment::Center);

    let mut navigation_row = Row::new().padding(3).spacing(10);

    // Start engine only if playing against engine
    navigation_row =
        navigation_row.push(Button::new(Text::new("Start engine")).on_press(Message::StartEngine));

    navigation_row =
        navigation_row.push(Button::new(Text::new("Undo move")).on_press(Message::UndoMove));

    navigation_row = navigation_row
        .push(Button::new(Text::new("< Previous")).on_press(Message::UndoMoveVirtual));

    navigation_row =
        navigation_row.push(Button::new(Text::new("Next >")).on_press(Message::NextMoveVirtual));

    navigation_row = navigation_row
        .push(Button::new(Text::new("Reset board")).on_press(Message::ResetBoardEngine));

    navigation_row = navigation_row
        .push(Button::new(Text::new("Show legal moves")).on_press(Message::PrintLegalMoves));

    navigation_row =
        navigation_row.push(Button::new(Text::new("Kiwipete")).on_press(Message::ChangeStartPos));

    navigation_row = navigation_row
        .push(Button::new(Text::new("Tournament")).on_press(Message::StartTournament));

    let mut moves_played = Row::new()
        .padding(3)
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);
    for (mut index, moves) in board.history.list.iter().enumerate() {
        if moves.next_move.data != 0 {
            index += 1;
            moves_played = moves_played.push(Text::new(
                index.to_string() + ". " + &moves.next_move.as_string(),
            ));
        }
    }

    let evaluation_bar = column![progress_bar(0.0..=100.0, eval)
        .vertical(true)
        .height(100 * 8)
        .width(10)
        .style(ProgressBar::Danger)]
    .padding(5);

    row![
        evaluation_bar,
        column![
            board_row,
            column![side_to_play, game_mode_row, navigation_row, moves_played]
                .width(board_height * 8)
                .height(Length::Fill)
                .align_items(Alignment::Center)
        ]
        .padding(5),
        if !engine_started {
            column![settings_tab]
        } else {
            column![timer_row]
        }
    ]
    .into()
}

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    // fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<Message, iced::Renderer<Theme>> {
        let column = Column::new()
            .spacing(20)
            .push(Text::new(self.title()).size(20))
            .push(self.content());

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(20)
            .into()
    }

    fn content(&self) -> Element<Message, iced::Renderer<Theme>>;
}

pub fn show_promotion_options() -> Element<'static, Message, iced::Renderer<Theme>> {
    Column::new()
        .push(Text::new("Select promotion piece:"))
        .push(
            Button::new(Svg::from_path(format!(
                "{}/pieces{}",
                env!("CARGO_MANIFEST_DIR"),
                "/wQ.svg"
            )))
            .on_press(Message::PromotionSelected(PromotionChoice::Queen)),
        )
        .push(
            Button::new(Svg::from_path(format!(
                "{}/pieces{}",
                env!("CARGO_MANIFEST_DIR"),
                "/wR.svg"
            )))
            .on_press(Message::PromotionSelected(PromotionChoice::Rook)),
        )
        .push(
            Button::new(Svg::from_path(format!(
                "{}/pieces{}",
                env!("CARGO_MANIFEST_DIR"),
                "/wB.svg"
            )))
            .on_press(Message::PromotionSelected(PromotionChoice::Bishop)),
        )
        .push(
            Button::new(Svg::from_path(format!(
                "{}/pieces{}",
                env!("CARGO_MANIFEST_DIR"),
                "/wN.svg"
            )))
            .on_press(Message::PromotionSelected(PromotionChoice::Knight)),
        )
        .into()
}
