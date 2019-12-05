use crate::rational::Rational;
use crate::visibility::Light;
use crate::Event;
use direction::{CardinalDirection, Direction};
pub use ecs::Entity;
use ecs::{ecs_components, ComponentTable, Ecs};
use grid_2d::{Coord, Grid, Size};
use line_2d::InfiniteStepIter;
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};
use shadowcast::vision_distance::Circle;
use std::time::Duration;
use vector::Radial;

fn period_per_frame(num_per_frame: u32) -> Duration {
    FRAME_DURATION / num_per_frame
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Carpet,
    Bullet,
    Smoke,
    ExplosionFlame,
    FormerHuman,
    Human,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Floor,
    Feature,
    Character,
    Particle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub coord: Coord,
    pub layer: Layer,
}

impl Location {
    fn new(coord: Coord, layer: Layer) -> Self {
        Self { coord, layer }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OnCollision {
    Explode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,
    Afraid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Npc {
    pub disposition: Disposition,
}

ecs_components! {
    components {
        location: Location,
        tile: Tile,
        opacity: u8,
        solid: (),
        realtime_movement: RealtimeComponent<Movement>,
        realtime_particle_emitter: RealtimeComponent<ParticleEmitter>,
        realtime_fade: RealtimeComponent<Fade>,
        realtime_light_colour_fade: RealtimeComponent<LightColourFade>,
        realtime: (),
        blocks_gameplay: (),
        light: Light,
        on_collision: OnCollision,
        colour_hint: Rgb24,
        npc: Npc,
    }
}
use components::Components;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightColourFade {
    fade: Fade,
    from: Rgb24,
    to: Rgb24,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeComponent<S> {
    state: S,
    until_next_tick: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    path: InfiniteStepIter,
    cardinal_period: Duration,
    ordinal_period: Duration,
}

struct Particle {
    movement: Option<Movement>,
    fade: Fade,
    tile: Tile,
    colour_hint: Option<Rgb24>,
    light: Option<Light>,
    light_colour_fade: Option<LightColourFade>,
    particle_emitter: Option<Box<ParticleEmitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleAngleRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DurationRange {
    min: Duration,
    max: Duration,
}

impl DurationRange {
    fn choose<R: Rng>(&self, rng: &mut R) -> Duration {
        rng.gen_range(self.min, self.max)
    }
}

impl ParticleAngleRange {
    fn all() -> Self {
        Self {
            min: -::std::f64::consts::PI,
            max: ::std::f64::consts::PI,
        }
    }
    fn choose<R: Rng>(&self, rng: &mut R) -> f64 {
        rng.gen_range(self.min, self.max)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleMovementSpec {
    angle_range: ParticleAngleRange,
    cardinal_period_range: DurationRange,
}

impl ParticleMovementSpec {
    fn movement<R: Rng>(&self, rng: &mut R) -> Movement {
        const VECTOR_LENGTH: f64 = 1000.;
        let angle_radians = self.angle_range.choose(rng);
        let radial = Radial {
            angle_radians,
            length: VECTOR_LENGTH,
        };
        let delta = radial.to_cartesian().to_coord_round_nearest();
        let path = InfiniteStepIter::new(delta);
        let cardinal_period = self.cardinal_period_range.choose(rng);
        Movement::new(path, cardinal_period)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum ParticleInitialFadeProgress {
    Zero,
    FromEmitter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleFadeSpec {
    initial_progress: ParticleInitialFadeProgress,
    full_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleLightColourFadeSpec {
    fade_spec: ParticleFadeSpec,
    from: Rgb24,
    to: Rgb24,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParticleColourSpec {
    from: Rgb24,
    to: Rgb24,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleLightSpec {
    chance: Rational,
    light: Light,
}

impl ParticleLightSpec {
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<Light> {
        if self.chance.roll(rng) {
            Some(self.light.clone())
        } else {
            None
        }
    }
}

impl ParticleColourSpec {
    fn choose<R: Rng>(self, rng: &mut R) -> Rgb24 {
        self.from.linear_interpolate(self.to, rng.gen())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterSpec {
    chance: Rational,
    particle_emitter: Box<ParticleEmitter>,
}

impl ParticleEmitterSpec {
    fn choose<R: Rng>(&self, rng: &mut R) -> Option<Box<ParticleEmitter>> {
        if self.chance.roll(rng) {
            Some(self.particle_emitter.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    period: Duration,
    movement_spec: Option<ParticleMovementSpec>,
    fade_spec: ParticleFadeSpec,
    tile: Tile,
    colour_spec: Option<ParticleColourSpec>,
    light_spec: Option<ParticleLightSpec>,
    light_colour_fade_spec: Option<ParticleLightColourFadeSpec>,
    particle_emitter_spec: Option<ParticleEmitterSpec>,
}

impl ParticleEmitter {
    fn emit<R: Rng>(&self, emitter_fade_progress: Option<FadeProgress>, rng: &mut R) -> Particle {
        let fade = match self.fade_spec.initial_progress {
            ParticleInitialFadeProgress::Zero => Fade::new(self.fade_spec.full_duration),
            ParticleInitialFadeProgress::FromEmitter => {
                Fade::new_with_progress(self.fade_spec.full_duration, emitter_fade_progress.unwrap_or_default())
            }
        };
        let light_colour_fade = self.light_colour_fade_spec.as_ref().map(|spec| {
            let fade = match spec.fade_spec.initial_progress {
                ParticleInitialFadeProgress::Zero => Fade::new(self.fade_spec.full_duration),
                ParticleInitialFadeProgress::FromEmitter => {
                    Fade::new_with_progress(self.fade_spec.full_duration, emitter_fade_progress.unwrap_or_default())
                }
            };
            LightColourFade {
                fade,
                from: spec.from,
                to: spec.to,
            }
        });
        Particle {
            movement: self.movement_spec.as_ref().map(|s| s.movement(rng)),
            fade,
            tile: self.tile,
            colour_hint: self.colour_spec.map(|c| c.choose(rng)),
            light: self.light_spec.as_ref().and_then(|l| l.choose(rng)),
            light_colour_fade,
            particle_emitter: self.particle_emitter_spec.as_ref().and_then(|p| p.choose(rng)),
        }
    }
    fn tick<R: Rng>(&self, emitter_fade_progress: Option<FadeProgress>, rng: &mut R) -> Tick<Particle> {
        Tick {
            data: self.emit(emitter_fade_progress, rng),
            duration: self.period,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum FadeProgress {
    Fading(u8),
    Complete,
}

impl FadeProgress {
    fn fading(self) -> Option<u8> {
        match self {
            Self::Fading(progress) => Some(progress),
            Self::Complete => None,
        }
    }
    fn is_complete(self) -> bool {
        match self {
            Self::Fading(_) => false,
            Self::Complete => true,
        }
    }
}

impl Default for FadeProgress {
    fn default() -> Self {
        Self::Fading(0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Fade {
    progress: FadeProgress,
    period: Duration,
}

impl Fade {
    fn new(duration: Duration) -> Self {
        Self::new_with_progress(duration, FadeProgress::default())
    }
    fn new_with_progress(full_duration: Duration, progress: FadeProgress) -> Self {
        let period = full_duration / 256;
        Self { progress, period }
    }
    fn tick(&mut self) -> Tick<FadeProgress> {
        self.progress = match self.progress {
            FadeProgress::Complete => FadeProgress::Complete,
            FadeProgress::Fading(progress) => match progress.checked_add(1) {
                Some(progress) => FadeProgress::Fading(progress),
                None => FadeProgress::Complete,
            },
        };
        Tick {
            data: self.progress,
            duration: self.period,
        }
    }
}

enum LightColourFadeProgress {
    Colour(Rgb24),
    Complete,
}

impl LightColourFade {
    fn tick(&mut self) -> Tick<LightColourFadeProgress> {
        let Tick {
            data: fade_progress,
            duration,
        } = self.fade.tick();
        let data = match fade_progress {
            FadeProgress::Complete => LightColourFadeProgress::Complete,
            FadeProgress::Fading(fading) => {
                LightColourFadeProgress::Colour(self.from.linear_interpolate(self.to, fading))
            }
        };
        Tick { data, duration }
    }
}

struct Tick<T> {
    data: T,
    duration: Duration,
}

impl Movement {
    fn ordinal_duration_from_cardinal_duration(duration: Duration) -> Duration {
        const SQRT_2_X_1_000_000: u64 = 1_414_214;
        let ordinal_micros = (duration.as_micros() as u64 * SQRT_2_X_1_000_000) / 1_000_000;
        Duration::from_micros(ordinal_micros)
    }

    fn new(path: InfiniteStepIter, cardinal_period: Duration) -> Self {
        Self {
            path,
            cardinal_period,
            ordinal_period: Self::ordinal_duration_from_cardinal_duration(cardinal_period),
        }
    }

    fn tick(&mut self) -> Tick<Direction> {
        let direction = self.path.step();
        let duration = if direction.is_cardinal() {
            self.cardinal_period
        } else {
            self.ordinal_period
        };
        Tick {
            data: direction,
            duration,
        }
    }
}

#[derive(Debug)]
struct OccupiedBy(Entity);

#[derive(Debug, Serialize, Deserialize)]
struct SpatialCell {
    floor: Option<Entity>,
    feature: Option<Entity>,
    character: Option<Entity>,
}

impl Default for SpatialCell {
    fn default() -> Self {
        Self {
            floor: None,
            feature: None,
            character: None,
        }
    }
}

enum SelectFieldMut<'a> {
    Tracked(&'a mut Option<Entity>),
    Untracked,
}

impl SpatialCell {
    fn select_field_mut(&mut self, layer: Layer) -> SelectFieldMut {
        match layer {
            Layer::Character => SelectFieldMut::Tracked(&mut self.character),
            Layer::Feature => SelectFieldMut::Tracked(&mut self.feature),
            Layer::Floor => SelectFieldMut::Tracked(&mut self.floor),
            Layer::Particle => SelectFieldMut::Untracked,
        }
    }
    fn insert(&mut self, entity: Entity, layer: Layer) -> Result<(), OccupiedBy> {
        let layer_field = match self.select_field_mut(layer) {
            SelectFieldMut::Tracked(layer_field) => layer_field,
            SelectFieldMut::Untracked => return Ok(()),
        };
        if let Some(&occupant) = layer_field.as_ref() {
            Err(OccupiedBy(occupant))
        } else {
            *layer_field = Some(entity);
            Ok(())
        }
    }
    fn clear(&mut self, layer: Layer) -> Option<Entity> {
        match self.select_field_mut(layer) {
            SelectFieldMut::Tracked(field) => field.take(),
            SelectFieldMut::Untracked => None,
        }
    }
}

fn location_insert(
    entity: Entity,
    location: Location,
    location_component: &mut ComponentTable<Location>,
    spatial_grid: &mut Grid<SpatialCell>,
) -> Result<(), OccupiedBy> {
    let cell = spatial_grid.get_checked_mut(location.coord);
    cell.insert(entity, location.layer)?;
    if let Some(original_location) = location_component.insert(entity, location) {
        let original_cell = spatial_grid.get_checked_mut(original_location.coord);
        let should_match_entity = original_cell.clear(original_location.layer);
        debug_assert_eq!(
            should_match_entity,
            Some(entity),
            "Current location of entity doesn't contain entity in spatial grid"
        );
    }
    Ok(())
}

fn is_solid_feature_at_coord(
    coord: Coord,
    solid_component: &ComponentTable<()>,
    spatial_grid: &Grid<SpatialCell>,
) -> bool {
    let cell = spatial_grid.get_checked(coord);
    if let Some(feature) = cell.feature {
        solid_component.contains(feature)
    } else {
        false
    }
}

struct RealtimeComponents<'a> {
    movement: Option<&'a mut RealtimeComponent<Movement>>,
    particle_emitter: Option<&'a mut RealtimeComponent<ParticleEmitter>>,
    fade: Option<&'a mut RealtimeComponent<Fade>>,
    light_colour_fade: Option<&'a mut RealtimeComponent<LightColourFade>>,
}

struct RealtimeTick {
    movement: Option<Direction>,
    particle_emitter: Option<Particle>,
    fade: Option<FadeProgress>,
    light_colour_fade: Option<LightColourFadeProgress>,
}

impl<'a> RealtimeComponents<'a> {
    fn tick<R: Rng>(&mut self, frame_remaining: Duration, rng: &mut R) -> Tick<RealtimeTick> {
        let mut until_tick = frame_remaining;
        if let Some(movement) = self.movement.as_ref() {
            until_tick = until_tick.min(movement.until_next_tick);
        }
        if let Some(particle_emitter) = self.particle_emitter.as_ref() {
            until_tick = until_tick.min(particle_emitter.until_next_tick);
        }
        if let Some(fade) = self.fade.as_ref() {
            until_tick = until_tick.min(fade.until_next_tick);
        }
        if let Some(light_colour_fade) = self.light_colour_fade.as_ref() {
            until_tick = until_tick.min(light_colour_fade.until_next_tick);
        }
        let movement = if let Some(movement) = self.movement.as_mut() {
            if until_tick == movement.until_next_tick {
                let tick = movement.state.tick();
                movement.until_next_tick = tick.duration;
                Some(tick.data)
            } else {
                movement.until_next_tick -= until_tick;
                None
            }
        } else {
            None
        };
        let fade = if let Some(fade) = self.fade.as_mut() {
            if until_tick == fade.until_next_tick {
                let tick = fade.state.tick();
                fade.until_next_tick = tick.duration;
                Some(tick.data)
            } else {
                fade.until_next_tick -= until_tick;
                None
            }
        } else {
            None
        };
        let light_colour_fade = if let Some(light_colour_fade) = self.light_colour_fade.as_mut() {
            if until_tick == light_colour_fade.until_next_tick {
                let tick = light_colour_fade.state.tick();
                light_colour_fade.until_next_tick = tick.duration;
                Some(tick.data)
            } else {
                light_colour_fade.until_next_tick -= until_tick;
                None
            }
        } else {
            None
        };
        let particle_emitter = if let Some(particle_emitter) = self.particle_emitter.as_mut() {
            if until_tick == particle_emitter.until_next_tick {
                let fade_progress = self.fade.as_ref().map(|f| f.state.progress);
                let tick = particle_emitter.state.tick(fade_progress, rng);
                particle_emitter.until_next_tick = tick.duration;
                Some(tick.data)
            } else {
                particle_emitter.until_next_tick -= until_tick;
                None
            }
        } else {
            None
        };
        Tick {
            duration: until_tick,
            data: RealtimeTick {
                movement,
                particle_emitter,
                fade,
                light_colour_fade,
            },
        }
    }
}

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    ecs: Ecs<Components>,
    spatial_grid: Grid<SpatialCell>,
    realtime_entities: Vec<Entity>,
}

impl World {
    pub fn new(size: Size) -> Self {
        let ecs = Ecs::new();
        let spatial_grid = Grid::new_default(size);
        Self {
            ecs,
            spatial_grid,
            realtime_entities: Vec::new(),
        }
    }
    pub fn spawn_player(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Player);
        self.ecs.components.light.insert(
            entity,
            Light {
                colour: Rgb24::new(255, 187, 127),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
        entity
    }
    pub fn spawn_former_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::FormerHuman);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Hostile,
            },
        );
        entity
    }
    pub fn spawn_human(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Character),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Human);
        self.ecs.components.npc.insert(
            entity,
            Npc {
                disposition: Disposition::Afraid,
            },
        );
        entity
    }

    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Floor),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Floor);
        entity
    }
    pub fn spawn_carpet(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Floor),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Carpet);
        entity
    }
    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Feature),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.tile.insert(entity, Tile::Wall);
        self.ecs.components.solid.insert(entity, ());
        self.ecs.components.opacity.insert(entity, 255);
        entity
    }
    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        let entity = self.ecs.create();
        location_insert(
            entity,
            Location::new(coord, Layer::Feature),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.light.insert(
            entity,
            Light {
                colour,
                vision_distance: Circle::new_squared(420),
                diminish: Rational {
                    numerator: 1,
                    denominator: 25,
                },
            },
        );
        entity
    }
    pub fn character_walk_in_direction(&mut self, entity: Entity, direction: CardinalDirection) {
        let current_location = self.ecs.components.location.get_mut(entity).unwrap();
        debug_assert_eq!(current_location.layer, Layer::Character);
        let target_coord = current_location.coord + direction.coord();
        if is_solid_feature_at_coord(target_coord, &self.ecs.components.solid, &self.spatial_grid) {
            return;
        }
        let target_location = Location::new(target_coord, Layer::Character);
        if let Err(OccupiedBy(_occupant)) = location_insert(
            entity,
            target_location,
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        ) {
            // TODO melee
        }
    }
    pub fn character_fire_bullet(&mut self, character: Entity, target: Coord) {
        let character_coord = self.ecs.components.location.get(character).unwrap().coord;
        if character_coord == target {
            return;
        }
        let bullet_entity = self.ecs.create();
        location_insert(
            bullet_entity,
            Location::new(character_coord, Layer::Particle),
            &mut self.ecs.components.location,
            &mut self.spatial_grid,
        )
        .unwrap();
        self.ecs.components.realtime.insert(bullet_entity, ());
        self.ecs.components.blocks_gameplay.insert(bullet_entity, ());
        self.ecs.components.realtime_movement.insert(
            bullet_entity,
            RealtimeComponent {
                state: Movement::new(
                    InfiniteStepIter::new(target - character_coord),
                    Duration::from_millis(16),
                ),
                until_next_tick: Duration::from_millis(0),
            },
        );
        self.ecs.components.realtime_particle_emitter.insert(
            bullet_entity,
            RealtimeComponent {
                state: ParticleEmitter {
                    period: Duration::from_micros(500),
                    movement_spec: Some(ParticleMovementSpec {
                        angle_range: ParticleAngleRange::all(),
                        cardinal_period_range: DurationRange {
                            min: Duration::from_millis(200),
                            max: Duration::from_millis(500),
                        },
                    }),
                    fade_spec: ParticleFadeSpec {
                        initial_progress: ParticleInitialFadeProgress::Zero,
                        full_duration: Duration::from_millis(1000),
                    },
                    tile: Tile::Smoke,
                    colour_spec: None,
                    light_spec: None,
                    light_colour_fade_spec: None,
                    particle_emitter_spec: None,
                },
                until_next_tick: Duration::from_millis(0),
            },
        );
        self.ecs.components.tile.insert(bullet_entity, Tile::Bullet);
        self.ecs
            .components
            .on_collision
            .insert(bullet_entity, OnCollision::Explode);
        self.ecs.components.light.insert(
            bullet_entity,
            Light {
                colour: Rgb24::new(255, 187, 63),
                vision_distance: Circle::new_squared(90),
                diminish: Rational {
                    numerator: 1,
                    denominator: 10,
                },
            },
        );
    }
    pub fn opacity(&self, coord: Coord) -> u8 {
        self.spatial_grid
            .get(coord)
            .and_then(|c| c.feature)
            .and_then(|e| self.ecs.components.opacity.get(e).cloned())
            .unwrap_or(0)
    }
    pub fn entity_coord(&self, entity: Entity) -> Coord {
        self.ecs.components.location.get(entity).unwrap().coord
    }
    pub fn entity_npc(&self, entity: Entity) -> &Npc {
        self.ecs.components.npc.get(entity).unwrap()
    }
    pub fn size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        !self.ecs.components.blocks_gameplay.is_empty()
    }
    fn spawn_explosion_emitter(
        ecs: &mut Ecs<Components>,
        spatial_grid: &mut Grid<SpatialCell>,
        coord: Coord,
        duration: Duration,
        num_particles_per_frame: u32,
        min_step: Duration,
        max_step: Duration,
        fade_duration: Duration,
    ) {
        let emitter_entity = ecs.entity_allocator.alloc();
        location_insert(
            emitter_entity,
            Location::new(coord, Layer::Particle),
            &mut ecs.components.location,
            spatial_grid,
        )
        .unwrap();
        ecs.components.realtime_fade.insert(
            emitter_entity,
            RealtimeComponent {
                state: Fade::new(duration),
                until_next_tick: Duration::from_millis(0),
            },
        );
        ecs.components.realtime.insert(emitter_entity, ());
        ecs.components.realtime_particle_emitter.insert(
            emitter_entity,
            RealtimeComponent {
                state: ParticleEmitter {
                    period: period_per_frame(num_particles_per_frame),
                    movement_spec: Some(ParticleMovementSpec {
                        angle_range: ParticleAngleRange::all(),
                        cardinal_period_range: DurationRange {
                            min: min_step,
                            max: max_step,
                        },
                    }),
                    fade_spec: ParticleFadeSpec {
                        initial_progress: ParticleInitialFadeProgress::FromEmitter,
                        full_duration: fade_duration,
                    },
                    tile: Tile::ExplosionFlame,
                    colour_spec: Some(ParticleColourSpec {
                        from: Rgb24::new(255, 255, 63),
                        to: Rgb24::new(255, 127, 0),
                    }),
                    light_spec: None,
                    light_colour_fade_spec: None,
                    particle_emitter_spec: Some(ParticleEmitterSpec {
                        chance: Rational {
                            numerator: 1,
                            denominator: 20,
                        },
                        particle_emitter: Box::new(ParticleEmitter {
                            period: min_step,
                            movement_spec: Some(ParticleMovementSpec {
                                angle_range: ParticleAngleRange::all(),
                                cardinal_period_range: DurationRange {
                                    min: Duration::from_millis(200),
                                    max: Duration::from_millis(500),
                                },
                            }),
                            fade_spec: ParticleFadeSpec {
                                initial_progress: ParticleInitialFadeProgress::Zero,
                                full_duration: Duration::from_millis(1000),
                            },
                            tile: Tile::Smoke,
                            colour_spec: None,
                            light_spec: None,
                            light_colour_fade_spec: None,
                            particle_emitter_spec: None,
                        }),
                    }),
                },
                until_next_tick: Duration::from_millis(0),
            },
        );
        ecs.components.light.insert(
            emitter_entity,
            Light {
                colour: Rgb24::new(255, 187, 63),
                vision_distance: Circle::new_squared(420),
                diminish: Rational {
                    numerator: 1,
                    denominator: 100,
                },
            },
        );
        ecs.components.realtime_light_colour_fade.insert(
            emitter_entity,
            RealtimeComponent {
                state: LightColourFade {
                    fade: Fade::new(fade_duration),
                    from: Rgb24::new(255, 187, 63),
                    to: Rgb24::new(0, 0, 0),
                },
                until_next_tick: Duration::from_millis(0),
            },
        );
    }
    fn explosion(ecs: &mut Ecs<Components>, spatial_grid: &mut Grid<SpatialCell>, coord: Coord) {
        Self::spawn_explosion_emitter(
            ecs,
            spatial_grid,
            coord,
            Duration::from_millis(250),
            50,
            Duration::from_millis(10),
            Duration::from_millis(30),
            Duration::from_millis(250),
        );
    }

    fn animation_movement(
        ecs: &mut Ecs<Components>,
        spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        movement_direction: Direction,
        events: &mut Vec<Event>,
    ) {
        if let Some(current_location) = ecs.components.location.get_mut(entity) {
            let next_coord = current_location.coord + movement_direction.coord();
            if is_solid_feature_at_coord(next_coord, &ecs.components.solid, spatial_grid) {
                if let Some(on_collision) = ecs.components.on_collision.get(entity) {
                    let current_coord = current_location.coord;
                    match on_collision {
                        OnCollision::Explode => {
                            events.push(Event::Explosion(current_coord));
                            Self::explosion(ecs, spatial_grid, current_coord);
                        }
                    }
                }
                ecs.remove(entity);
            } else {
                current_location.coord += movement_direction.coord();
            }
        } else {
            ecs.remove(entity);
        }
    }

    fn animation_emit_particle(
        ecs: &mut Ecs<Components>,
        spatial_grid: &mut Grid<SpatialCell>,
        mut particle: Particle,
        coord: Coord,
    ) {
        let particle_entity = ecs.entity_allocator.alloc();
        if let Some(movement) = particle.movement.take() {
            ecs.components.realtime_movement.insert(
                particle_entity,
                RealtimeComponent {
                    until_next_tick: movement.cardinal_period,
                    state: movement,
                },
            );
        }
        location_insert(
            particle_entity,
            Location::new(coord, Layer::Particle),
            &mut ecs.components.location,
            spatial_grid,
        )
        .unwrap();
        ecs.components.tile.insert(particle_entity, particle.tile);
        ecs.components.realtime_fade.insert(
            particle_entity,
            RealtimeComponent {
                state: particle.fade,
                until_next_tick: Duration::from_millis(0),
            },
        );
        ecs.components.realtime.insert(particle_entity, ());
        if let Some(colour_hint) = particle.colour_hint {
            ecs.components.colour_hint.insert(particle_entity, colour_hint);
        }
        if let Some(light) = particle.light.take() {
            ecs.components.light.insert(particle_entity, light);
        }
        if let Some(light_colour_fade) = particle.light_colour_fade.take() {
            ecs.components.realtime_light_colour_fade.insert(
                particle_entity,
                RealtimeComponent {
                    state: light_colour_fade,
                    until_next_tick: Duration::from_millis(0),
                },
            );
        }
        if let Some(particle_emitter) = particle.particle_emitter.take() {
            ecs.components.realtime_particle_emitter.insert(
                particle_entity,
                RealtimeComponent {
                    state: *particle_emitter,
                    until_next_tick: Duration::from_millis(0),
                },
            );
        }
    }

    fn animation_fade(ecs: &mut Ecs<Components>, progress: FadeProgress, entity: Entity) {
        if progress.is_complete() {
            ecs.remove(entity);
        }
    }

    fn animation_light_colour_fade(ecs: &mut Ecs<Components>, progress: LightColourFadeProgress, entity: Entity) {
        match progress {
            LightColourFadeProgress::Colour(colour) => {
                if let Some(light) = ecs.components.light.get_mut(entity) {
                    light.colour = colour;
                }
            }
            LightColourFadeProgress::Complete => {
                ecs.components.light.remove(entity);
            }
        }
    }

    fn animation_tick_single_entity(
        ecs: &mut Ecs<Components>,
        spatial_grid: &mut Grid<SpatialCell>,
        entity: Entity,
        mut realtime_tick: RealtimeTick,
        events: &mut Vec<Event>,
    ) {
        if let Some(movement_direction) = realtime_tick.movement {
            Self::animation_movement(ecs, spatial_grid, entity, movement_direction, events);
        }
        if let Some(particle) = realtime_tick.particle_emitter.take() {
            if let Some(location) = ecs.components.location.get(entity) {
                let coord = location.coord;
                Self::animation_emit_particle(ecs, spatial_grid, particle, coord);
            }
        }
        if let Some(progress) = realtime_tick.fade {
            Self::animation_fade(ecs, progress, entity);
        }
        if let Some(progress) = realtime_tick.light_colour_fade {
            Self::animation_light_colour_fade(ecs, progress, entity);
        }
    }
    pub fn animation_tick<R: Rng>(&mut self, events: &mut Vec<Event>, rng: &mut R) {
        self.realtime_entities.extend(self.ecs.components.realtime.entities());
        for entity in self.realtime_entities.drain(..) {
            let mut frame_remaining = FRAME_DURATION;
            while frame_remaining > Duration::from_micros(0) {
                let mut realtime_components = RealtimeComponents {
                    movement: self.ecs.components.realtime_movement.get_mut(entity),
                    particle_emitter: self.ecs.components.realtime_particle_emitter.get_mut(entity),
                    fade: self.ecs.components.realtime_fade.get_mut(entity),
                    light_colour_fade: self.ecs.components.realtime_light_colour_fade.get_mut(entity),
                };
                let Tick {
                    duration,
                    data: realtime_tick,
                } = realtime_components.tick(frame_remaining, rng);
                frame_remaining -= duration;
                Self::animation_tick_single_entity(
                    &mut self.ecs,
                    &mut self.spatial_grid,
                    entity,
                    realtime_tick,
                    events,
                );
            }
        }
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        let tile_component = &self.ecs.components.tile;
        let location_component = &self.ecs.components.location;
        let realtime_fade_component = &self.ecs.components.realtime_fade;
        let colour_hint_component = &self.ecs.components.colour_hint;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            if let Some(location) = location_component.get(entity) {
                let fade = realtime_fade_component
                    .get(entity)
                    .and_then(|f| f.state.progress.fading());
                let colour_hint = colour_hint_component.get(entity).cloned();
                Some(ToRenderEntity {
                    coord: location.coord,
                    layer: location.layer,
                    tile,
                    fade,
                    colour_hint,
                })
            } else {
                None
            }
        })
    }
    pub fn lights<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &'a Light)> {
        self.ecs.components.light.iter().filter_map(move |(entity, light)| {
            self.ecs
                .components
                .location
                .get(entity)
                .map(|location| (location.coord, light))
        })
    }
    pub fn contains_wall(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.feature {
                self.ecs.components.tile.get(entity) == Some(&Tile::Wall)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn contains_npc(&self, coord: Coord) -> bool {
        if let Some(spatial_cell) = self.spatial_grid.get(coord) {
            if let Some(entity) = spatial_cell.character {
                self.ecs.components.npc.contains(entity)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn character_at(&self, coord: Coord) -> Option<Entity> {
        self.spatial_grid.get(coord).and_then(|cell| cell.character)
    }
}

pub struct ToRenderEntity {
    pub coord: Coord,
    pub layer: Layer,
    pub tile: Tile,
    pub fade: Option<u8>,
    pub colour_hint: Option<Rgb24>,
}
