use bevy::prelude::*;

use crate::resources::Mute;

#[derive(Resource, Default)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub throttle: bool,
    pub fire: bool,
    pub ok: bool,
    pub weapon_1: bool,
    pub weapon_2: bool,
    pub weapon_3: bool,
    pub weapon_4: bool,
    pub weapon_next: bool,
    pub weapon_prev: bool,
}

pub fn update_input_state(
    mut state: ResMut<InputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut mute: ResMut<Mute>,
) {
    state.left = keyboard.pressed(KeyCode::KeyA);
    state.right = keyboard.pressed(KeyCode::KeyD);
    state.up = keyboard.just_pressed(KeyCode::KeyW);
    state.down = keyboard.just_pressed(KeyCode::KeyS);
    state.throttle = keyboard.pressed(KeyCode::KeyW);
    state.fire = keyboard.pressed(KeyCode::Space);
    state.ok = keyboard.just_pressed(KeyCode::Space);
    state.weapon_1 = keyboard.just_pressed(KeyCode::Digit1);
    state.weapon_2 = keyboard.just_pressed(KeyCode::Digit2);
    state.weapon_3 = keyboard.just_pressed(KeyCode::Digit3);
    state.weapon_4 = keyboard.just_pressed(KeyCode::Digit4);
    state.weapon_next = keyboard.just_pressed(KeyCode::KeyE);
    state.weapon_prev = keyboard.just_pressed(KeyCode::KeyQ);

    for gamepad in &gamepads {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        state.left |= gamepad.pressed(GamepadButton::DPadLeft) || left_stick_x < 0.1;
        state.right |= gamepad.pressed(GamepadButton::DPadRight) || left_stick_x > 0.1;
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        state.up |= gamepad.pressed(GamepadButton::DPadUp) || left_stick_y > 0.1;
        state.down |= gamepad.pressed(GamepadButton::DPadDown) || left_stick_y < 0.1;
        state.throttle |=
            gamepad.pressed(GamepadButton::South) || gamepad.pressed(GamepadButton::LeftTrigger);
        state.fire |=
            gamepad.pressed(GamepadButton::West) || gamepad.pressed(GamepadButton::RightTrigger);
        state.ok |= gamepad.just_pressed(GamepadButton::West)
            || gamepad.just_pressed(GamepadButton::RightTrigger);
        state.weapon_next |= gamepad.pressed(GamepadButton::RightTrigger2);
        state.weapon_prev |= gamepad.pressed(GamepadButton::LeftTrigger2);
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        mute.enabled = !mute.enabled;
    }
}
