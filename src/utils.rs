pub fn lerp(start: f32, end: f32, position: f32) -> f32 {
    start + (end - start) * position.clamp(0.0, 1.0)
}
