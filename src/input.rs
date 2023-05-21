use bevy::prelude::*;

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
    keyboard: Res<Input<KeyCode>>,
    gamepads: Res<Gamepads>,
    buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
) {
    state.left = keyboard.pressed(KeyCode::A);
    state.right = keyboard.pressed(KeyCode::D);
    state.up = keyboard.just_pressed(KeyCode::W);
    state.down = keyboard.just_pressed(KeyCode::S);
    state.throttle = keyboard.pressed(KeyCode::W);
    state.fire = keyboard.pressed(KeyCode::Space);
    state.ok = keyboard.just_pressed(KeyCode::Space);
    state.weapon_1 = keyboard.just_pressed(KeyCode::Key1);
    state.weapon_2 = keyboard.just_pressed(KeyCode::Key2);
    state.weapon_3 = keyboard.just_pressed(KeyCode::Key3);
    state.weapon_4 = keyboard.just_pressed(KeyCode::Key4);
    state.weapon_next = keyboard.just_pressed(KeyCode::E);
    state.weapon_prev = keyboard.just_pressed(KeyCode::Q);

    for gamepad in gamepads.iter() {
        let left_stick_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap_or(0.0);
        state.left |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadLeft))
            || left_stick_x < 0.1;
        state.right |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadRight))
            || left_stick_x > 0.1;
        let left_stick_y = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
            .unwrap_or(0.0);
        state.up |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadUp))
            || left_stick_y > 0.1;
        state.down |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadDown))
            || left_stick_y < 0.1;
        state.throttle |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
            || buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger));
        state.fire |= buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
            || buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::RightTrigger));
        state.ok |= buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
            || buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::RightTrigger));
        state.weapon_next |= buttons.pressed(GamepadButton::new(
            gamepad,
            GamepadButtonType::RightTrigger2,
        ));
        state.weapon_prev |=
            buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::LeftTrigger2));
    }
}
