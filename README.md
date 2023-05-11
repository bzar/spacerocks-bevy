# Space Rocks! (bevy rewrite)

[Original Space Rocks! here](https://github.com/bzar/spacerocks)

## TODO

- [x] Bevy 0.10 port
- [x] Backgrounds
- [x] Asteroids
- [x] Ship
  - [x] Movement
  - [x] Sprite
- [x] Weapons
  - [x] Rapid
  - [x] Spread
  - [x] Beam
  - [x] Plasma
- [x] UFOs
- [x] Shield
- [x] Powerups
- [x] Game HUD
- [x] Collisions
  - [x] Weapon -> Asteroid
  - [x] Weapon -> UFO
  - [x] Asteroid -> Ship
  - [x] Asteroid -> Asteroid
  - [x] UFO -> Ship
  - [x] Powerup -> Ship
  - [x] Invulnerability
- [x] Levels
  - [x] Change
  - [x] Progress
  - [x] UFO logic
- [x] Explosions
- [x] Ship respawn delay
- [x] Level start delay
- [x] Level start text animation
- [x] Particle effects
  - [ ] Additive alpha blending
- [x] Score/powerup notifications
- [ ] Music
- [ ] Sounds
  - [ ] Game sounds
  - [ ] Title sounds
- [x] Title
- [x] Game over
- [ ] High Score
- [x] View letterboxing
- [ ] Gamepad controls

### Known bugs

- Beam weapon collision does not match the graphics
- Sometimes a Ufo spawns multiple Powerups
  - Probably related to the number of projectiles hitting the UFO at the same time
- Asteroid despawns sometimes cause multiple despawn errors
  - Probably related to the number of projectiles hitting the asteroid at the same time
- Some entities are not despawned on level change (projectiles)
  - Add LevelEntity
- Shield doesn't help sometimes
  - Probably doesn't add asteroid/ufo speed to bounce
