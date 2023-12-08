//! A raycasting backend for [`bevy_sprite`].

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy::{prelude::*, render::primitives::Aabb, window::PrimaryWindow};
use bevy_mod_picking::{
    backend::{HitData, PointerHits},
    picking_core::{PickSet, Pickable},
    pointer::{PointerId, PointerLocation},
};

#[derive(Clone)]
pub struct AabbPickingBackend;

impl Plugin for AabbPickingBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, aabb_picking.in_set(PickSet::Backend));
    }
}

pub fn aabb_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    sprite_query: Query<(Entity, &Aabb, &GlobalTransform, Option<&Pickable>)>,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_aabbs: Vec<_> = sprite_query.iter().collect();
    sorted_aabbs.sort_by(|a, b| {
        (b.2.translation().z)
            .partial_cmp(&a.2.translation().z)
            .unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform)) = cameras
            .iter()
            .filter(|(_, camera, _)| camera.is_active)
            .find(|(_, camera, _)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
        else {
            continue;
        };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let picks: Vec<(Entity, HitData)> = sorted_aabbs
            .iter()
            .copied()
            .filter_map(|(entity, aabb, sprite_transform, pickable, ..)| {
                if blocked {
                    return None;
                }

                let rect = Rect::from_center_half_size(Vec2::ZERO, aabb.half_extents.xy());

                // Transform cursor pos to sprite coordinate system
                let cursor_pos_sprite = sprite_transform
                    .affine()
                    .inverse()
                    .transform_point3((cursor_pos_world, 0.0).into());

                let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());
                blocked =
                    is_cursor_in_sprite && pickable.map(|p| p.should_block_lower) != Some(false);

                is_cursor_in_sprite.then_some((
                    entity,
                    HitData::new(cam_entity, sprite_transform.translation().z, None, None),
                ))
            })
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order))
    }
}
