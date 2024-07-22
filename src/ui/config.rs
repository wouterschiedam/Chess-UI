use crate::defs::Square;

#[derive(Debug, Clone)]
pub struct UIConfig {
    pub show_coordinates: bool,
    pub flip_board: bool,
    pub search_depth: u32,
    pub game_mode: GameMode,
    pub player_side: u32,
}

impl ::std::default::Default for UIConfig {
    fn default() -> Self {
        Self {
            show_coordinates: true,
            flip_board: false,
            search_depth: 3,
            game_mode: GameMode::PlayerPlayer,
            player_side: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Promotions {
    pub promotion: Option<PromotionChoice>,
    pub show_promotion_prompt: bool,
    pub promotion_square: Option<Square>,
}

impl Promotions {
    pub fn default() -> Self {
        Self {
            promotion: None,
            show_promotion_prompt: false,
            promotion_square: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PromotionChoice {
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GameMode {
    PlayerPlayer,
    PlayerEngine,
    EngineEngine,
}

impl GameMode {
    pub const ALL: [GameMode; 3] = [
        GameMode::PlayerPlayer,
        GameMode::PlayerEngine,
        GameMode::EngineEngine,
    ];
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameMode::PlayerPlayer => "Player vs Player",
                GameMode::PlayerEngine => "Player vs Engine",
                GameMode::EngineEngine => "Engine vs Engine",
            }
        )
    }
}
