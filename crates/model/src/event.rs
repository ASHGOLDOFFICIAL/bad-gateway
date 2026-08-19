/// Set of events that are captured from user.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserGameEvent {
    /// User tries to move in direction given in vector.
    SetMove(glam::Vec2),

    /// User tries to aim in direction given in vector.
    SetAim(glam::Vec2),

    /// User tries to fire ranged weapons, at the given scale.
    Fire(f32),

    /// User tries to engage thrusters for a speed boost.
    Boost(f32),
}
