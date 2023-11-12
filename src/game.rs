/* Game logic implementation file */

use crate::linmath;
pub type Vec2 = linmath::Vec2<f32>;

#[derive(Copy, Clone)]
pub struct Player {
    pub position: Vec2,
    pub health: f32,
} /* Player */

#[derive(Copy, Clone)]
pub struct Enemy {
    pub position: Vec2,
    pub health: f32,
} /* Enemy */

#[derive(Copy, Clone)]
pub struct Bullet {
    pub position: Vec2,
    pub direction: Vec2,
} /* Bullet */

pub struct Engine {
    pub player: Player,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
} /* Engine */

pub const PLAYER_SIZE: f32 = 1.0;
pub const ENEMY_SIZE: f32 = 3.0;
pub const BULLET_SIZE: f32 = 0.5;

impl Engine {
    pub fn new() -> Engine {
        Engine {
            player: Player { position: Vec2::new(30.0, 47.0), health: 100.0 },
            enemies: Vec::<Enemy>::new(),
            bullets: Vec::<Bullet>::new(),
        }
    } /* new */

    pub fn update(&mut self, delta_time: f32) {
        // Update bullets and enemies positions
        for bullet in &mut self.bullets {
            bullet.position += bullet.direction * delta_time;
        }
        for enemy in &mut self.enemies {
            enemy.position += (self.player.position - enemy.position).normalized() * delta_time;
        }

        // Intersect player with enemies
        for enemy in &self.enemies {
            const MIN_INTERSECTION_DISTANCE: f32 = PLAYER_SIZE * PLAYER_SIZE + ENEMY_SIZE * ENEMY_SIZE;
            let distance = (enemy.position - self.player.position).length2();

            if distance < MIN_INTERSECTION_DISTANCE {
                self.player.health -= 5.0;
            }
        }
        // Intersect enemies with bullets
        let new_enemy_list: Vec<Enemy> = self.enemies
            .iter()
            .filter(|enemy| {
                for bullet in &self.bullets {
                    const MIN_INTERSECTION_DISTANCE: f32 = ENEMY_SIZE * ENEMY_SIZE + BULLET_SIZE * BULLET_SIZE;
                    let distance = (enemy.position - bullet.position).length2();

                    if distance < MIN_INTERSECTION_DISTANCE {
                        return false
                    }
                }

                return true
            })
            .cloned()
            .collect();
        self.enemies = new_enemy_list;
    } /* update */
} /* impl Engine */
