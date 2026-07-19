use std::time::Duration;

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{components::LocalStorage, game::{Board, BoardPos, Card, DECK_SIZE, DepotRole, RANKS, SettingsState, Skin, Suit}};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ActionRecord {
    Move { pos1: BoardPos, pos2: BoardPos, rev: bool },
    Combine { card1: Card, card2: Card },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScreenState {
    #[default] Game, 
    Settings, Help,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub deal: Vec<Card>,
    #[serde(skip)]
    pub animation_key: AnimationKey, // used for syncing and to provide animator components with cycling keys
    pub history: Vec<ActionRecord>,
    pub undo_stack: Vec<usize>,
    pub already_won: bool,
    pub num_wins: i32,

    pub screen_state: ScreenState,

    pub allow_undo: bool,
    pub skin: Skin,
}

impl GameState {
    pub fn new_deal(rng: &mut impl Rng) -> Vec<Card> {
        let mut deck = Vec::with_capacity(DECK_SIZE);
        for rank in RANKS {
            for suit in Suit::iter_normal() {
                deck.push(Card { rank, suit });
            }
        }

        deck.shuffle(rng);
        deck
    }

    pub fn init() -> Self {
        let mut res = Self {
            board: Board::empty(),
            deal: vec![],
            animation_key: 0,
            history: vec![],
            undo_stack: vec![],
            already_won: false,
            num_wins: 0,
            screen_state: ScreenState::Game,
            allow_undo: true,
            skin: Skin::default(),
        };

        res.new_game();
        res
    }

    pub fn new_game(&mut self) {
        let deal = Self::new_deal(&mut rand::rng());
        self.board = Board::from_deal(&deal);
        self.deal = deal;
        self.history.clear();
        self.undo_stack.clear();
        self.already_won = false;

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn is_busy(&self) -> bool {
        self.is_acting()
    }

    pub fn is_acting(&self) -> bool {
        !self.board.animation_acts.is_empty()
    }

    pub fn undo_possible(&self) -> bool {
        self.allow_undo && !self.undo_stack.is_empty()
    }

    fn do_move_raw(&mut self, pos1: BoardPos, pos2: BoardPos, rev: bool) {
        self.board.do_move(pos1, pos2, rev);
        self.history.push(ActionRecord::Move { pos1, pos2, rev })
    }

    pub fn can_stack(&self, back: Card, front: Card) -> bool {
        (back.suit != front.suit || back.suit == Suit::Wild) && 
        back.rank.abs_diff(front.rank) == 1
    }

    pub fn can_select(&self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }
        let slice = &self.board.depots[depot][ord..];

        let Some(role) = DepotRole::role(depot) else { return false };
        match role {
            DepotRole::Tableau => slice.windows(2).all(|w| self.can_stack(w[0], w[1])),
            DepotRole::Foundation => false,
            DepotRole::EngineIn => slice.len() <= 1,
            DepotRole::EngineOut => slice.len() <= 1,
        }
    }

    pub fn move_intent(&mut self, pos1: BoardPos, pos2: BoardPos, rev: bool) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];
        let num_moved = depot1.len() - pos1.card_index;
        if pos2.card_index != depot2.len() { return false; }

        let Some(role) = DepotRole::role(pos2.depot_index) else { return false };
        let history_len = self.history.len();
        match role {
            DepotRole::Tableau => {
                let card = if !rev { depot1[pos1.card_index] } else { *depot1.last().unwrap() };
                let ok = depot2.last().is_none_or(|&c| self.can_stack(c, card));
                if !ok { return false; }
                self.do_move_raw(pos1, pos2, rev);
            },
            DepotRole::Foundation => {
                let slice = &depot1[pos1.card_index ..];
                let sort_rank = depot2.last().map(|c| c.rank).unwrap_or(0) + 1;
                if !slice.iter().rev().map(|c| c.rank).eq((sort_rank..).take(slice.len())) {
                    return false;
                }
                self.do_move_raw(pos1, pos2, true);
            }
            DepotRole::EngineIn => {
                if num_moved != 1 || !depot2.is_empty() { return false; }
                self.do_move_raw(pos1, pos2, rev);
            }
            DepotRole::EngineOut => return false,
        }

        self.undo_stack.push(history_len);
        true
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if self.is_over() { return; }

        if let Some(src) = self.board.selected {
            if pos == src { 
                self.board.selected = None; 
                return;
            }
            if src.depot_index == pos.depot_index && self.can_select(pos) {
                self.board.selected = Some(pos);
                return;
            }

            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            self.move_intent(src, dest, false);
        } else {
            if self.can_select(pos) {
                self.board.selected = Some(pos);
            }
        }
    }

    // right-click is shortcut for reverse-stacking
    pub fn oncontextmenu(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if self.is_over() { return; }

        if let Some(src) = self.board.selected {
            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            self.move_intent(src, dest, true);
        }
    }

    pub fn ondoubleclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if self.is_over() { return; }
        if !self.can_select(pos) { return; } // needed, or illegal stacks can still be moved this way!

        self.move_intent(pos, self.board.top_pos(DepotRole::Foundation.id(0)), false);
    }

    pub fn is_won(&self) -> bool {
        self.board.depots[DepotRole::Foundation.id(0)].len() == 26
    }

    pub fn is_over(&self) -> bool {
        self.is_won()
    }

    pub fn check_auto_moves(&mut self) {
        if self.is_busy() { return; }
        if self.is_over() { return; }

        use DepotRole::*;
        if !self.board.depots[EngineOut.id(0)].is_empty() { return; }
        let (Some(&card1), Some(&card2)) = 
            (self.board.depots[EngineIn.id(0)].last(), self.board.depots[EngineIn.id(1)].last()) else {return};
        if card1.suit == Suit::Wild || card2.suit == Suit::Wild { return; }
        self.board.do_combine();
        self.history.push(ActionRecord::Combine { card1, card2 });
    }

    pub fn advance_animations(&mut self, key: AnimationKey) {
        if key != self.animation_key { return; }
        self.animation_key = self.animation_key.wrapping_add(1);
        
        self.board.advance_actions();

        if self.is_won() {
            if !self.already_won {
                self.num_wins += 1;
                self.already_won = true;
            }
        } else {
            self.check_auto_moves();
        }

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn undo(&mut self) {
        if self.is_busy() || !self.undo_possible() { return; }
        let Some(target_len) = self.undo_stack.pop() else {return};
        while self.history.len() > target_len {
            let rec = self.history.pop().unwrap();
            match rec {
                ActionRecord::Move { pos1, pos2, rev } => {
                    self.board.do_move(pos2, pos1, rev)
                },
                ActionRecord::Combine { card1, card2 } => {
                    self.board.undo_combine(card1, card2);
                },
            }
            self.board.advance_actions(); // no animation, as repeated card moves on same card causes problems
        }
        LocalStorage.save_game_state(&self);
    }

    pub fn restart(&mut self) {
        if self.history.is_empty() || !self.undo_possible() { return; }
        self.board = Board::from_deal(&self.deal);
        self.history.clear();
        self.undo_stack.clear();
        LocalStorage.save_game_state(&self);
    }

    pub fn new_settings_state(&self) -> SettingsState {
        SettingsState {
            allow_undo: self.allow_undo,
            skin: self.skin,
        }
    }

    pub fn apply_settings(&mut self, settings: &SettingsState){
        self.allow_undo = settings.allow_undo;
        self.skin = settings.skin;
        LocalStorage.save_game_state(&self);
    }
}