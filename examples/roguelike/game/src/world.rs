use direction::{CardinalDirection, Direction};
pub use ecs::Entity;
use ecs::{ecs_components, ComponentTable, Ecs};
use grid_2d::{Coord, Grid, Size};
use line_2d::InfiniteStepIter;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Carpet,
    Bullet,
    Smoke,
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

ecs_components! {
    components {
        location: Location,
        tile: Tile,
        opacity: u8,
        solid: (),
        realtime_movement: RealtimeComponent<Movement>,
        realtime_particle_emitter: RealtimeComponent<ParticleEmitter>,
        realtime_fade: RealtimeComponent<Fade>,
        realtime: (),
    }
}
use components::Components;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fade {
    progress: Option<u8>,
    period: Duration,
}

impl Fade {
    fn new(duration: Duration) -> Self {
        let period = duration / 256;
        Self {
            progress: Some(0),
            period,
        }
    }
    fn tick(&mut self) -> Tick<Option<u8>> {
        self.progress = self.progress.and_then(|progress| progress.checked_add(1));
        Tick {
            data: self.progress,
            duration: self.period,
        }
    }
}

impl ParticleEmitter {
    fn tick(&mut self) -> Tick<()> {
        Tick {
            data: (),
            duration: self.period,
        }
    }
}

struct Tick<T> {
    data: T,
    duration: Duration,
}

impl Movement {
    fn ordinal_duration_from_cardinal_duration(duration: Duration) -> Duration {
        const SQRT_2_X_1_000_000: u64 = 1414214;
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
}

struct RealtimeTick {
    movement: Option<Direction>,
    particle_emitter: Option<()>,
    fade: Option<Option<u8>>,
}

impl<'a> RealtimeComponents<'a> {
    fn tick(&mut self, frame_remaining: Duration) -> Tick<RealtimeTick> {
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
        let particle_emitter = if let Some(particle_emitter) = self.particle_emitter.as_mut() {
            if until_tick == particle_emitter.until_next_tick {
                let tick = particle_emitter.state.tick();
                particle_emitter.until_next_tick = tick.duration;
                Some(tick.data)
            } else {
                particle_emitter.until_next_tick -= until_tick;
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
        Tick {
            duration: until_tick,
            data: RealtimeTick {
                movement,
                particle_emitter,
                fade,
            },
        }
    }
}

const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    ecs: Ecs<Components>,
    spatial_grid: Grid<SpatialCell>,
    entity_buffer: Vec<Entity>,
}

impl World {
    pub fn new(size: Size) -> Self {
        let ecs = Ecs::new();
        let spatial_grid = Grid::new_default(size);
        let entity_buffer = Vec::new();
        Self {
            ecs,
            spatial_grid,
            entity_buffer,
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
        self.ecs.components.realtime_movement.insert(
            bullet_entity,
            RealtimeComponent {
                state: Movement::new(
                    InfiniteStepIter::new(target - character_coord),
                    Duration::from_millis(32),
                ),
                until_next_tick: Duration::from_millis(0),
            },
        );
        self.ecs.components.realtime_particle_emitter.insert(
            bullet_entity,
            RealtimeComponent {
                state: ParticleEmitter {
                    period: Duration::from_millis(16),
                },
                until_next_tick: Duration::from_millis(0),
            },
        );
        self.ecs.components.tile.insert(bullet_entity, Tile::Bullet);
    }
    pub fn entity_coord(&self, entity: Entity) -> Coord {
        self.ecs.components.location.get(entity).unwrap().coord
    }
    pub fn size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn has_pending_animation(&self) -> bool {
        !self.ecs.components.realtime.is_empty()
    }
    pub fn animation_tick<R: Rng>(&mut self, _rng: &mut R) {
        let to_remove = &mut self.entity_buffer;
        let mut fade_buffer = Vec::new();
        for (entity, ()) in self.ecs.components.realtime.iter() {
            let mut realtime_components = RealtimeComponents {
                movement: self.ecs.components.realtime_movement.get_mut(entity),
                particle_emitter: self.ecs.components.realtime_particle_emitter.get_mut(entity),
                fade: self.ecs.components.realtime_fade.get_mut(entity),
            };
            let mut frame_remaining = FRAME_DURATION;
            while frame_remaining > Duration::from_micros(0) {
                let Tick {
                    duration,
                    data: realtime_tick,
                } = realtime_components.tick(frame_remaining);
                frame_remaining -= duration;
                if let Some(movement_direction) = realtime_tick.movement.as_ref() {
                    if let Some(current_location) = self.ecs.components.location.get_mut(entity) {
                        current_location.coord += movement_direction.coord();
                        if is_solid_feature_at_coord(
                            current_location.coord,
                            &self.ecs.components.solid,
                            &self.spatial_grid,
                        ) {
                            to_remove.push(entity);
                            break;
                        }
                    } else {
                        to_remove.push(entity);
                        break;
                    }
                }
                if realtime_tick.particle_emitter.is_some() {
                    if let Some(location) = self.ecs.components.location.get(entity) {
                        let particle_entity = self.ecs.entity_allocator.alloc();
                        location_insert(
                            particle_entity,
                            Location::new(location.coord, Layer::Particle),
                            &mut self.ecs.components.location,
                            &mut self.spatial_grid,
                        )
                        .unwrap();
                        self.ecs.components.tile.insert(particle_entity, Tile::Smoke);
                        fade_buffer.push((
                            particle_entity,
                            RealtimeComponent {
                                state: Fade::new(Duration::from_millis(1000)),
                                until_next_tick: Duration::from_millis(0),
                            },
                        ));
                    }
                }
                if let Some(maybe_progress) = realtime_tick.fade.as_ref() {
                    if maybe_progress.is_none() {
                        to_remove.push(entity);
                    }
                }
            }
        }
        for entity in to_remove.drain(..) {
            self.ecs.remove(entity);
        }
        for (entity, fade) in fade_buffer.drain(..) {
            self.ecs.components.realtime_fade.insert(entity, fade);
            self.ecs.components.realtime.insert(entity, ());
        }
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        let tile_component = &self.ecs.components.tile;
        let location_component = &self.ecs.components.location;
        let realtime_fade_component = &self.ecs.components.realtime_fade;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            if let Some(location) = location_component.get(entity) {
                let fade = realtime_fade_component.get(entity).and_then(|f| f.state.progress);
                Some(ToRenderEntity {
                    coord: location.coord,
                    layer: location.layer,
                    tile: tile,
                    fade,
                })
            } else {
                None
            }
        })
    }
}

pub struct ToRenderEntity {
    pub coord: Coord,
    pub layer: Layer,
    pub tile: Tile,
    pub fade: Option<u8>,
}
