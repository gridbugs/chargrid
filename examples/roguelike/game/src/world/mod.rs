use crate::{visibility::Light, ExternalEvent};
use ecs::{Entity, EntityAllocator};
use grid_2d::{Coord, Size};
use rand::Rng;
use rgb24::Rgb24;
use serde::{Deserialize, Serialize};

mod spatial;
use spatial::Spatial;

mod data;
use data::{Components, Npc};
pub use data::{Disposition, EntityData, HitPoints, Layer, Location, Tile};

mod realtime_periodic;
pub use realtime_periodic::animation::{Context as AnimationContext, FRAME_DURATION as ANIMATION_FRAME_DURATION};
use realtime_periodic::data::RealtimeComponents;

mod query;

mod explosion;
pub use explosion::spec as explosion_spec;

mod action;

mod spawn;
pub use spawn::make_player;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub realtime_components: RealtimeComponents,
    pub spatial: Spatial,
}

impl World {
    pub fn new(size: Size) -> Self {
        let entity_allocator = EntityAllocator::default();
        let components = Components::default();
        let realtime_components = RealtimeComponents::default();
        let spatial = Spatial::new(size);
        Self {
            entity_allocator,
            components,
            realtime_components,
            spatial,
        }
    }
}

impl World {
    pub fn to_render_entities<'a>(&'a self) -> impl 'a + Iterator<Item = ToRenderEntity> {
        let tile_component = &self.components.tile;
        let spatial = &self.spatial;
        let realtime_fade_component = &self.realtime_components.fade;
        let colour_hint_component = &self.components.colour_hint;
        let blood_component = &self.components.blood;
        let ignore_lighting_component = &self.components.ignore_lighting;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            if let Some(location) = spatial.location(entity) {
                let fade = realtime_fade_component.get(entity).and_then(|f| f.state.fading());
                let colour_hint = colour_hint_component.get(entity).cloned();
                let blood = blood_component.contains(entity);
                let ignore_lighting = ignore_lighting_component.contains(entity);
                Some(ToRenderEntity {
                    coord: location.coord,
                    layer: location.layer,
                    tile,
                    fade,
                    colour_hint,
                    blood,
                    ignore_lighting,
                })
            } else {
                None
            }
        })
    }

    pub fn all_lights_by_coord<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &'a Light)> {
        self.components
            .light
            .iter()
            .filter_map(move |(entity, light)| self.spatial.coord(entity).map(|&coord| (coord, light)))
    }

    pub fn character_info(&self, entity: Entity) -> Option<CharacterInfo> {
        let &coord = self.spatial.coord(entity)?;
        let &hit_points = self.components.hit_points.get(entity)?;
        Some(CharacterInfo { coord, hit_points })
    }
}

impl World {
    pub fn entity_coord(&self, entity: Entity) -> Option<Coord> {
        self.spatial.coord(entity).cloned()
    }
    pub fn entity_npc(&self, entity: Entity) -> &Npc {
        self.components.npc.get(entity).unwrap()
    }
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entity_allocator.exists(entity)
    }
    pub fn size(&self) -> Size {
        self.spatial.grid_size()
    }
    pub fn is_gameplay_blocked(&self) -> bool {
        !self.components.blocks_gameplay.is_empty()
    }
    pub fn animation_tick<R: Rng>(
        &mut self,
        animation_context: &mut AnimationContext,
        external_events: &mut Vec<ExternalEvent>,
        rng: &mut R,
    ) {
        animation_context.tick(self, external_events, rng)
    }
    pub fn clone_entity_data(&self, entity: Entity) -> EntityData {
        self.components.clone_entity_data(entity)
    }
}

pub struct ToRenderEntity {
    pub coord: Coord,
    pub layer: Option<Layer>,
    pub tile: Tile,
    pub fade: Option<u8>,
    pub colour_hint: Option<Rgb24>,
    pub blood: bool,
    pub ignore_lighting: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterInfo {
    pub coord: Coord,
    pub hit_points: HitPoints,
}
