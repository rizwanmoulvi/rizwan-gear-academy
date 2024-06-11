use gstd::{prelude::*, msg};
use gtest::{Program, System};
use pebbles_game_io::{PebblesInit, PebblesAction, PebblesEvent, GameState, Player, DifficultyLevel};

#[test]
fn test_init() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

    program.send(1, init_msg);

    let state: GameState = program.read_state().unwrap();
    assert_eq!(state.pebbles_count, 15);
    assert_eq!(state.max_pebbles_per_turn, 2);
}

#[test]
fn test_user_turn() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

        program.send(1, init_msg);

    let action = PebblesAction::Turn(1);
    program.send(1, action);

    let event: PebblesEvent = program.read_message().unwrap();
    match event {
        PebblesEvent::CounterTurn(pebbles) => {
            assert!(pebbles <= 2 && pebbles >= 1);
        }
        _ => panic!("Unexpected event"),
    }

    let state: GameState = program.read_state().unwrap();
    assert_eq!(state.pebbles_remaining, 14 - 1 - 1);
}

#[test]
fn test_program_turn() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

    program.send(1, init_msg);

    let state: GameState = program.read_state().unwrap();
    if let Player::Program = state.first_player {
        let event: PebblesEvent = program.read_message().unwrap();
        match event {
            PebblesEvent::CounterTurn(pebbles) => {
                assert!(pebbles <= 2 && pebbles >= 1);
            }
            _ => panic!("Unexpected event"),
        }
    }
}

#[test]
fn test_restart_game() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

    program.send(1, init_msg);

    let action = PebblesAction::Turn(1);
    program.send(1, action);

    let restart_msg = PebblesAction::Restart {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 20,
        max_pebbles_per_turn: 3,
    };
    program.send(1, restart_msg);

    let state: GameState = program.read_state().unwrap();
    assert_eq!(state.pebbles_count, 20);
    assert_eq!(state.max_pebbles_per_turn, 3);
    assert_eq!(state.pebbles_remaining, 20);
    assert_eq!(state.difficulty, DifficultyLevel::Hard);
}

#[test]
fn test_invalid_turn() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

    program.send(1, init_msg);

    let invalid_action = PebblesAction::Turn(3);
    let result = program.try_send(1, invalid_action);
    assert!(result.is_err());
}

#[test]
fn test_give_up() {
    let sys = System::new();
    let program = Program::current(&sys);

    let init_msg = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };

    program.send(1, init_msg);

    let action = PebblesAction::GiveUp;
    program.send(1, action);

    let event: PebblesEvent = program.read_message().unwrap();
    assert_eq!(event, PebblesEvent::Won(Player::Program));

    let state: GameState = program.read_state().unwrap();
    assert_eq!(state.winner, Some(Player::Program));
}

