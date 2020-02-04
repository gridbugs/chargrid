use crate::{visibility::Light, ExternalEvent};
use direction::CardinalDirection;
use ecs::{Ecs, Entity};
use grid_2d::{Coord, Grid, Size};
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};

mod spatial_grid;
use spatial_grid::SpatialGrid;

mod data;
use data::{Components, Npc};
pub use data::{Disposition, Layer, Tile};

mod realtime_periodic;
pub use realtime_periodic::animation::Context as AnimationContext;
use realtime_periodic::data::RealtimeComponents;

mod query;
pub use query::ToRenderEntity;

mod explosion;
pub use explosion::spec as explosion_spec;

mod action;
mod spawn;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    ecs: Ecs<Components>,
    realtime_components: RealtimeComponents,
    spatial_grid: SpatialGrid,
}

impl World {
    pub fn new(size: Size) -> Self {
        let ecs = Ecs::new();
        let realtime_components = RealtimeComponents::default();
        let spatial_grid = Grid::new_default(size);
        Self {
            ecs,
            realtime_components,
            spatial_grid,
        }
    }
}

impl World {
    pub fn spawn_player(&mut self, coord: Coord) -> Entity {
        spawn::player(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_former_human(&mut self, coord: Coord) -> Entity {
        spawn::former_human(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_human(&mut self, coord: Coord) -> Entity {
        spawn::human(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_floor(&mut self, coord: Coord) -> Entity {
        spawn::floor(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_carpet(&mut self, coord: Coord) -> Entity {
        spawn::carpet(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_wall(&mut self, coord: Coord) -> Entity {
        spawn::wall(&mut self.ecs, &mut self.spatial_grid, coord)
    }
    pub fn spawn_light(&mut self, coord: Coord, colour: Rgb24) -> Entity {
        spawn::light(&mut self.ecs, &mut self.spatial_grid, coord, colour)
    }
}

impl World {
    pub fn contains_wall(&self, coord: Coord) -> bool {
        query::is_wall_at_coord(&self.ecs, &self.spatial_grid, coord)
    }
    pub fn contains_npc(&self, coord: Coord) -> bool {
        query::is_npc_at_coord(&self.ecs, &self.spatial_grid, coord)
    }
    pub fn character_at(&self, coord: Coord) -> Option<Entity> {
        query::get_character_at_coord(&self.spatial_grid, coord)
    }
    pub fn opacity(&self, coord: Coord) -> u8 {
        query::get_opacity_at_coord(&self.ecs, &self.spatial_grid, coord)
    }
    pub fn lights<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &'a Light)> {
        query::all_lights_by_coord(&self.ecs)
    }
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        query::all_entites_to_render(&self.ecs, &self.realtime_components)
    }
}

impl World {
    pub fn character_walk_in_direction(&mut self, entity: Entity, direction: CardinalDirection) {
        action::character_walk_in_direction(&mut self.ecs, &mut self.spatial_grid, entity, direction)
    }
    pub fn character_fire_rocket(&mut self, character: Entity, target: Coord) {
        action::character_fire_rocket(
            &mut self.ecs,
            &mut self.realtime_components,
            &mut self.spatial_grid,
            character,
            target,
        )
    }
}

impl World {
    pub fn entity_coord(&self, entity: Entity) -> Option<Coord> {
        self.ecs.components.location.get(entity).map(|l| l.coord)
    }
    pub fn entity_npc(&self, entity: Entity) -> &Npc {
        self.ecs.components.npc.get(entity).unwrap()
    }
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.ecs.entity_allocator.exists(entity)
    }
    pub fn size(&self) -> Size {
        self.spatial_grid.size()
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        !self.ecs.components.blocks_gameplay.is_empty()
    }
    pub fn animation_tick<R: Rng>(
        &mut self,
        animation_context: &mut AnimationContext,
        external_events: &mut Vec<ExternalEvent>,
        rng: &mut R,
    ) {
        animation_context.tick(
            &mut self.ecs,
            &mut self.realtime_components,
            &mut self.spatial_grid,
            external_events,
            rng,
        )
    }
}
