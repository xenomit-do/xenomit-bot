use super::*;

/// Ore `typeID` rendered as palladium.
pub const PALLADIUM_TYPE_ID: i32 = 8;

/// Stable resource labels for the ore type IDs used by the client.
pub fn ore_tag(type_id: i32) -> Option<&'static str> {
    match type_id {
        0 => Some("Prometium"),
        1 => Some("Endurium"),
        2 => Some("Terbium"),
        4 => Some("Prometid"),
        5 => Some("Duranium"),
        6 => Some("Promerium"),
        7 => Some("Seprom"),
        PALLADIUM_TYPE_ID => Some("Palladium"),
        28 => Some("Rinusk"),
        _ => None,
    }
}

/// Resources are identified by their type, not by a per-instance name/ID.
pub fn entity_label(entity: &LiveEntity) -> Option<String> {
    match entity.kind {
        LiveKind::Ore => entity
            .has_type_id
            .then(|| ore_tag(entity.type_id).map(str::to_owned))
            .flatten(),
        LiveKind::Box => Some("Bonus Box".to_owned()),
        _ if entity.is_collectable => Some("Resource".to_owned()),
        _ => None,
    }
}

/// Enemy: a clan at war (`clan_diplomacy` 3), or a ship whose faction is
/// set and differs from the hero's.
pub fn is_enemy_ship(entity: &LiveEntity, hero_faction: i32) -> bool {
    entity.clan_diplomacy == 3 || (entity.faction != 0 && entity.faction != hero_faction)
}

/// Ally: shares the hero's faction, or a shared nonzero clan tag.
pub fn is_ally_ship(entity: &LiveEntity, hero_faction: i32, hero_clan: i32) -> bool {
    entity.faction == hero_faction || (entity.clan != 0 && entity.clan == hero_clan)
}

/// Marker diameter in px and color for one entity. The hero, pets and
/// safe-place kinds are painted separately.
pub fn marker_style(entity: &LiveEntity, hero_faction: i32, hero_clan: i32) -> (f32, u32) {
    match entity.kind {
        LiveKind::Box => (3.0, COLOR_BOX),
        LiveKind::Ore if entity.has_type_id && entity.type_id == PALLADIUM_TYPE_ID => {
            (3.0, COLOR_PALLADIUM)
        }
        LiveKind::Ore => (3.0, COLOR_ORE),
        LiveKind::Mine => (3.0, COLOR_MINE),
        LiveKind::Npc => (4.0, COLOR_NPC),
        LiveKind::Player => {
            if is_enemy_ship(entity, hero_faction) {
                (4.0, COLOR_ENEMY)
            } else if is_ally_ship(entity, hero_faction, hero_clan) {
                (4.0, COLOR_ALLY)
            } else {
                (4.0, COLOR_NEUTRAL)
            }
        }
        _ => (3.0, COLOR_OTHER),
    }
}
