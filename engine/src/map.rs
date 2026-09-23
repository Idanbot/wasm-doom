//! Hub-and-spoke level: "Site Nadir-7, upper works".
//!
//! Layout on the 48x32 cell grid (see `docs/map-authoring.md` for the
//! ASCII plan and the authoring guide):
//!
//! - Start Hangar (west) -> west lane -> Central Plaza (hub, vista)
//! - North lane -> Tech Lab (Scattergun, ambush closet A)
//! - East lane -> Chapel (Lance) -> doored Vault (boss seal arena)
//! - South lane -> Flesh Pit (Pyre, rising ambush)
//! - 3 secret closets (cell 9 doors): hangar cache, plaza vent, vault crypt
//!
//! Construction uses the generic carvers in `lib.rs`
//! (`room`/`hall_h`/`hall_v`/`pillar`/`mix_edge`); everything zone-specific
//! lives here so a second map is a new `build_*`/`place_*` pair, not
//! engine surgery.

use crate::consts::*;
use crate::enemies::is_hostile_kind;
use crate::types::AmbushTrigger;
use crate::Engine;

/// Player spawn: hangar floor, facing east into the west lane.
pub(crate) const PLAYER_START: (f32, f32, f32) = (4.5, 15.5, 0.0);

/// Boss arena spots inside the vault (plus player-relative fallbacks).
pub(crate) const BOSS_SPOTS: [(f32, f32); 2] = [(40.5, 24.5), (42.5, 26.5)];

/// One ambush: a player zone rect plus the group that spawns once.
pub(crate) struct AmbushDef {
    pub zone: (f32, f32, f32, f32),
    pub group: &'static [(u8, f32, f32)],
}

pub(crate) const AMBUSH_DEFS: &[AmbushDef] = &[
    // A-north closet: husks vent from the alcove east of the north lane.
    AmbushDef {
        zone: (21.5, 6.5, 24.5, 10.5),
        group: &[(EK_HUSK, 25.5, 7.6), (EK_HUSK, 24.6, 8.4)],
    },
    // A-east closet: brute + husk drop south of the east lane.
    AmbushDef {
        zone: (29.0, 14.5, 33.5, 16.5),
        group: &[(EK_BRUTE, 31.5, 17.5), (EK_HUSK, 30.5, 17.5)],
    },
    // Pit rise: wraiths lift off the far flesh while the Pyre sits center,
    // with Martyr drones floating up behind them.
    AmbushDef {
        zone: (20.5, 23.0, 24.5, 25.5),
        group: &[
            (EK_WRAITH, 23.5, 27.5),
            (EK_WRAITH, 20.5, 27.5),
            (EK_MARTYR, 21.5, 27.5),
            (EK_MARTYR, 22.5, 26.5),
        ],
    },
];

/// Carve the hub-and-spoke shell. Idempotent per fresh map fill.
pub(crate) fn build_hub_spoke(e: &mut Engine) {
    e.map.fill(1);
    e.floor.fill(0);
    e.decal.fill(0);

    // Start hangar (west), central plaza (hub), tech lab (north),
    // chapel (east), vault (boss arena, south-east), flesh pit (south).
    e.room(1, 12, 11, 8, 1, 0);
    e.room(16, 10, 14, 12, 1, 0);
    e.room(16, 1, 13, 6, 6, 0);
    e.room(33, 10, 9, 9, 2, 1);
    e.room(33, 19, 13, 11, 5, 1);
    e.room(16, 23, 13, 7, 3, 0);

    // Lanes stitch the hub to each zone; every lane carries a door pair.
    e.hall_h(11, 16, 15, 13);
    e.hall_v(22, 6, 10, 8);
    e.hall_h(29, 33, 15, 31);
    e.hall_v(22, 21, 23, 22);

    // Vault entrance: doored passage rendered with the seal texture
    // (see `is_boss_door`). Auto-opens on approach like any door.
    for (x, y) in [(36, 18), (37, 18), (36, 19), (37, 19)] {
        e.set_cell(x, y, 8);
    }

    // Ambush alcoves: open pockets in the wall mass beside the lanes.
    // Monsters spawn here only when their trigger fires (see below).
    for (x, y) in [(24, 7), (25, 7), (26, 7), (24, 8), (25, 8), (26, 8)] {
        e.set_cell(x, y, 0);
    }
    for (x, y) in [(30, 17), (31, 17), (32, 17), (30, 18), (31, 18), (32, 18)] {
        e.set_cell(x, y, 0);
    }

    // Secrets: cell-9 doors need USE (E); each one found bumps `secrets`.
    for (x, y) in [(4, 19), (5, 19)] {
        e.set_cell(x, y, 9);
    }
    for (x, y) in [(4, 20), (5, 20), (4, 21), (5, 21)] {
        e.set_cell(x, y, 0);
    }
    for (x, y) in [(29, 12), (29, 13)] {
        e.set_cell(x, y, 9);
    }
    for (x, y) in [(30, 12), (31, 12), (30, 13), (31, 13)] {
        e.set_cell(x, y, 0);
    }
    for (x, y) in [(33, 24), (33, 25)] {
        e.set_cell(x, y, 9);
    }
    for (x, y) in [(31, 24), (32, 24), (31, 25), (32, 25)] {
        e.set_cell(x, y, 0);
    }

    // Cover + landmarks: pipe block centers the plaza, pillars break
    // firing lines in the lab, chapel, vault and pit.
    e.pillar(22, 15, 4);
    e.pillar(18, 12, 1);
    e.pillar(26, 18, 1);
    e.pillar(24, 3, 6);
    e.pillar(37, 14, 2);
    e.pillar(35, 22, 5);
    e.pillar(42, 25, 5);
    e.pillar(19, 25, 3);
    e.pillar(24, 26, 3);

    // Chapel bone niches, hangar hazard strips.
    e.set_cell(33, 13, 5);
    e.set_cell(33, 16, 5);
    e.set_cell(41, 13, 5);
    e.set_cell(41, 16, 5);
    e.set_cell(1, 14, 7);
    e.set_cell(1, 16, 7);

    // Thematic wall variation per zone.
    e.mix_edge(1, 12, 11, 8, &[1, 7, 4, 1]);
    e.mix_edge(16, 10, 14, 12, &[1, 4, 1, 6, 4]);
    e.mix_edge(16, 1, 13, 6, &[6, 1, 6, 4, 6]);
    e.mix_edge(33, 10, 9, 9, &[2, 5, 2, 2, 5]);
    e.mix_edge(33, 19, 13, 11, &[5, 1, 5, 4, 5]);
    e.mix_edge(16, 23, 13, 7, &[3, 2, 3, 3]);

    // Seal flag floor under the vault arena.
    for j in SEAL_Y..SEAL_Y + SEAL_H {
        for i in SEAL_X..SEAL_X + SEAL_W {
            if e.cell(i, j) == 0 {
                let idx = j as usize * MAP_W + i as usize;
                if idx < e.floor.len() {
                    e.floor[idx] = 2;
                }
            }
        }
    }
}

/// Props, pickups and the opening hostile cast. Zone by zone so balance
/// passes read like a playlist: hangar calm, lanes warm, zones hot.
pub(crate) fn place_hub_spoke(e: &mut Engine) {
    e.ents = [crate::types::Ent {
        aim: 0.0,
        kind: 0,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        hp: 0,
        timer: 0.0,
        frame: 0.0,
        anim: ANIM_IDLE,
        anim_time: 0.0,
        anim_lock: 0.0,
        skin: SKIN_NONE,
        radius: 0.25,
        flash: 0.0,
        stun: 0.0,
        effect_tick: 0.0,
        zoff: 0.0,
    }; ENT_N];
    e.ambush = AMBUSH_DEFS
        .iter()
        .map(|d| AmbushTrigger { x0: d.zone.0, y0: d.zone.1, x1: d.zone.2, y1: d.zone.3, fired: false })
        .collect();

    // Hangar: safe start, one medkit, crates to strafe around.
    e.spawn(EK_MED, 3.5, 17.5);
    e.spawn(EK_LAMP, 3.5, 13.5);
    e.spawn(EK_LAMP, 8.5, 13.5);
    e.spawn(EK_CRATE, 6.5, 17.5);
    e.spawn(EK_CRATE, 9.5, 16.5);
    e.spawn(EK_FLAME, 5.5, 13.2);
    // West lane dressing.
    e.spawn(EK_CHAIN, 13.5, 14.2);
    e.spawn(EK_LAMP, 15.5, 15.5);
    // Plaza: landmark lamps, bait in the open (commit before the fight).
    e.spawn(EK_LAMP, 17.5, 11.5);
    e.spawn(EK_LAMP, 27.5, 11.5);
    e.spawn(EK_LAMP, 17.5, 19.5);
    e.spawn(EK_LAMP, 27.5, 19.5);
    e.spawn(EK_MED, 20.5, 15.5);
    e.spawn(EK_AMMO, 24.5, 15.5);
    e.spawn(EK_CRATE, 18.5, 18.5);
    e.spawn(EK_CHAIN, 22.5, 12.5);
    // North lane + tech lab: Scattergun behind the ambush.
    e.spawn(EK_CHAIN, 22.5, 9.3);
    e.spawn(EK_GUN2, 22.5, 3.5);
    e.spawn(EK_AMMO, 25.5, 4.5);
    e.spawn(EK_LAMP, 17.5, 2.5);
    e.spawn(EK_LAMP, 26.5, 4.5);
    // East lane + chapel: Lance before the long firing line.
    e.spawn(EK_CHAIN, 32.5, 16.5);
    e.spawn(EK_GUN4, 37.5, 12.5);
    e.spawn(EK_AMMO, 39.5, 15.5);
    e.spawn(EK_MED, 34.5, 16.5);
    e.spawn(EK_LAMP, 34.5, 11.5);
    e.spawn(EK_LAMP, 39.5, 11.5);
    e.spawn(EK_BARREL, 40.5, 16.5);
    // Vault: stocked for the seal ritual, barrels punish clustering.
    e.spawn(EK_MED, 43.5, 26.5);
    e.spawn(EK_AMMO, 39.5, 22.5);
    e.spawn(EK_BARREL, 35.5, 26.5);
    e.spawn(EK_BARREL, 42.5, 21.5);
    e.spawn(EK_LAMP, 40.5, 21.5);
    e.spawn(EK_CHAIN, 38.5, 24.5);
    // South lane + flesh pit: Pyre earns its doorway-holding fantasy.
    e.spawn(EK_FLAME, 22.5, 22.5);
    e.spawn(EK_GUN5, 22.5, 26.5);
    e.spawn(EK_BARREL, 18.5, 25.5);
    e.spawn(EK_BARREL, 20.5, 27.5);
    e.spawn(EK_MED, 26.5, 24.5);
    e.spawn(EK_AMMO, 19.5, 27.5);
    e.spawn(EK_LAMP, 22.5, 24.5);
    e.spawn(EK_FLAME, 26.5, 25.5);
    // Secrets reward the curious: armor cache, Ripper vent, vault crypt.
    e.spawn(EK_ARMOR, 4.5, 20.5);
    e.spawn(EK_GUN3, 30.5, 12.5);
    e.spawn(EK_AMMO, 31.5, 13.5);
    e.spawn(EK_ARMOR, 31.5, 24.5);
    e.spawn(EK_MED, 32.5, 25.5);

    e.spawn_hostiles(1);
}

/// Fire each ambush once when the player crosses its zone. Called from
/// `tick` while the run is live. The wall-rumble cue reuses `EV_DOOR`
/// (no HUD-ABI change for a new event bit).
pub(crate) fn check_ambushes(e: &mut Engine) {
    use crate::events::EV_DOOR;
    for i in 0..e.ambush.len() {
        let group = AMBUSH_DEFS.get(i).map(|d| d.group).unwrap_or(&[]);
        let (fired, inside) = {
            let t = match e.ambush.get(i) {
                Some(t) => t,
                None => continue,
            };
            (t.fired, e.px >= t.x0 && e.px <= t.x1 && e.py >= t.y0 && e.py <= t.y1)
        };
        if fired || !inside {
            continue;
        }
        if let Some(t) = e.ambush.get_mut(i) {
            t.fired = true;
        }
        let mut spawned = false;
        for &(kind, x, y) in group {
            if e.spawn(kind, x, y).is_some() {
                spawned = true;
            }
        }
        if spawned {
            e.events |= EV_DOOR;
        }
    }
}

/// Count live hostiles (test + tuning helper over the table flag).
#[allow(dead_code)]
pub(crate) fn living_hostiles(e: &Engine) -> usize {
    e.ents.iter().filter(|x| x.hp > 0 && is_hostile_kind(x.kind)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flood fill from the player start. Doors (8) and secret doors (9)
    /// count as passable: the former auto-open on approach, the latter
    /// open with USE, so both are traversable in a real run.
    fn reachable_cells(e: &Engine) -> Vec<bool> {
        let mut seen = vec![false; MAP_CELLS];
        let mut stack = vec![(PLAYER_START.0.floor() as i32, PLAYER_START.1.floor() as i32)];
        while let Some((x, y)) = stack.pop() {
            if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
                continue;
            }
            let idx = y as usize * MAP_W + x as usize;
            if seen[idx] {
                continue;
            }
            let c = e.map[idx];
            if c != 0 && c != 8 && c != 9 && c != 10 {
                continue;
            }
            seen[idx] = true;
            stack.push((x + 1, y));
            stack.push((x - 1, y));
            stack.push((x, y + 1));
            stack.push((x, y - 1));
        }
        seen
    }

    fn assert_reachable(seen: &[bool], x: f32, y: f32, what: &str) {
        let idx = y.floor() as usize * MAP_W + x.floor() as usize;
        assert!(seen[idx], "{what} at ({x}, {y}) is unreachable from spawn");
    }

    #[test]
    fn hub_spoke_zones_are_all_connected() {
        let e = Engine::new(160, 100);
        let seen = reachable_cells(&e);
        // Lanes, zone centers, weapon pickups, seal, secrets, ambush pockets.
        for (x, y, what) in [
            (22.5, 13.5, "plaza center"),
            (13.5, 15.5, "west lane"),
            (22.5, 9.5, "north lane"),
            (31.5, 15.5, "east lane"),
            (22.5, 22.5, "south lane"),
            (22.5, 3.5, "Scattergun"),
            (37.5, 12.5, "Lance"),
            (22.5, 26.5, "Pyre"),
            (41.5, 24.5, "seal arena"),
            (4.5, 20.5, "secret cache"),
            (30.5, 12.5, "secret vent"),
            (31.5, 24.5, "secret crypt"),
            (25.5, 7.6, "north closet"),
            (31.5, 17.5, "east closet"),
        ] {
            assert_reachable(&seen, x, y, what);
        }
    }

    #[test]
    fn ambush_triggers_fire_once_and_spawn_their_group() {
        let mut e = Engine::new(160, 100);
        assert_eq!(e.ambush.len(), AMBUSH_DEFS.len());
        let before = living_hostiles(&e);
        e.px = 22.5;
        e.py = 8.5;
        check_ambushes(&mut e);
        assert!(e.ambush[0].fired);
        assert_eq!(living_hostiles(&e), before + AMBUSH_DEFS[0].group.len());
        // Firing again changes nothing.
        check_ambushes(&mut e);
        assert_eq!(living_hostiles(&e), before + AMBUSH_DEFS[0].group.len());
    }

    #[test]
    fn all_three_secrets_count() {
        let mut e = Engine::new(160, 100);
        assert_eq!(e.secrets, 0);
        assert!(e.open_door_at(4, 19, true));
        assert!(e.open_door_at(29, 12, true));
        assert!(e.open_door_at(33, 24, true));
        assert_eq!(e.secrets, 3);
    }

    #[test]
    fn opening_cast_matches_the_zone_playlist() {
        let e = Engine::new(160, 100);
        // 14 HOSTILES plus 3 ambush groups staged (unfired).
        assert_eq!(living_hostiles(&e), 14);
        assert!(e.ambush.iter().all(|t| !t.fired));
    }
}
