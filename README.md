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
- [ ] Particle effects
- [ ] Score/powerup notifications
- [ ] Music
- [ ] Sounds
- [ ] Scene system
- [ ] Title
- [ ] Game over
- [ ] High Score
- [ ] View letterboxing

### Known bugs

- UI does not update correctly on some runs
- Beam weapon collision does not match the graphics
- Ufo lasers cause too big bounce with shield
- Beam sometimes gets stuck
- Sometimes a Ufo spawns multiple Powerups
- Asteroid despawns sometimes cause multiple despawn errors
- Some entities are not despawned on level change (projectiles)
