use gstd::{prelude::*, msg, exec};
use pebbles_game_io::{PebblesInit, PebblesAction, PebblesEvent, GameState, Player, DifficultyLevel};
use rand::Rng;

static mut GAME_STATE: Option<GameState> = None;

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[no_mangle]
pub extern "C" fn init() {
    let PebblesInit { difficulty, pebbles_count, max_pebbles_per_turn } = msg::load().expect("Failed to load init message");

    if pebbles_count == 0 || max_pebbles_per_turn == 0 || max_pebbles_per_turn > pebbles_count {
        panic!("Invalid game configuration");
    }

    let first_player = if get_random_u32() % 2 == 0 { Player::User } else { Player::Program };

    let mut game_state = GameState {
        pebbles_count,
        max_pebbles_per_turn,
        pebbles_remaining: pebbles_count,
        difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

    unsafe {
        GAME_STATE = Some(game_state.clone());
    }

    if let Player::Program = first_player {
        program_turn(&mut game_state);
    }
}

#[no_mangle]
pub extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to load handle message");

    let mut game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };

    match action {
        PebblesAction::Turn(pebbles) => {
            if pebbles == 0 || pebbles > game_state.max_pebbles_per_turn {
                panic!("Invalid number of pebbles");
            }

            game_state.pebbles_remaining = game_state.pebbles_remaining.saturating_sub(pebbles);
            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to reply");
                return;
            }

            program_turn(&mut game_state);
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply");
        }
        PebblesAction::Restart { difficulty, pebbles_count, max_pebbles_per_turn } => {
            game_state = GameState {
                difficulty,
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                first_player: if get_random_u32() % 2 == 0 { Player::User } else { Player::Program },
                winner: None,
            };
            if let Player::Program = game_state.first_player {
                program_turn(&mut game_state);
            }
        }
    }

    unsafe {
        GAME_STATE = Some(game_state);
    }
}

fn program_turn(game_state: &mut GameState) {
    let pebbles = match game_state.difficulty {
        DifficultyLevel::Easy => {
            let mut rng = rand::thread_rng();
            rng.gen_range(1..=game_state.max_pebbles_per_turn)
        }
        DifficultyLevel::Hard => {
            // Implementing a winning strategy here, if possible
            (game_state.pebbles_remaining % (game_state.max_pebbles_per_turn + 1)) as u32
        }
    };

    game_state.pebbles_remaining = game_state.pebbles_remaining.saturating_sub(pebbles);

    if game_state.pebbles_remaining == 0 {
        game_state.winner = Some(Player::Program);
        msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply");
    } else {
        msg::reply(PebblesEvent::CounterTurn(pebbles), 0).expect("Failed to reply");
    }
}

#[no_mangle]
pub extern "C" fn state() {
    let game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };
    msg::reply(game_state, 0).expect("Failed to reply");
}
