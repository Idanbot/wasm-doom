//! Hub-and-spoke level: "Site Nadir-7, upper works".
//!
//! Layout on the 48x32 cell grid (see `docs/map-authoring.md` for the
//! ASCII plan and the authoring guide):
//!
//! - Start Hangar (west) -> west lane -> Central Plaza (hub, vista)
//! - North lane -> Tech Lab (Scattergun, ambush closet A)
//! - East lane -> Chapel (Lance) -> doored Vault (boss override arena)
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
pub(crate) const LEVEL_COUNT: usize = 3;

const PLAYER_STARTS: [(f32, f32, f32); LEVEL_COUNT] =
    [PLAYER_START, (4.5, 6.5, 0.0), (4.5, 15.5, 0.0)];
const BOSS_SPOTS_BY_LEVEL: [[(f32, f32); 2]; LEVEL_COUNT] = [
    [(40.5, 24.5), (43.5, 27.5)],
    [(38.5, 23.5), (43.5, 25.5)],
    [(40.5, 13.5), (40.5, 18.5)],
];
const OVERRIDES: [(f32, f32); LEVEL_COUNT] = [(40.5, 21.5), (43.5, 27.5), (43.5, 15.5)];

pub(crate) fn level_index(wave: i32) -> usize {
    (wave.saturating_sub(1) as usize) % LEVEL_COUNT
}

pub(crate) fn player_start(wave: i32) -> (f32, f32, f32) {
    PLAYER_STARTS[level_index(wave)]
}

pub(crate) fn boss_spots(wave: i32) -> &'static [(f32, f32); 2] {
    &BOSS_SPOTS_BY_LEVEL[level_index(wave)]
}

pub(crate) fn override_point(wave: i32) -> (f32, f32) {
    OVERRIDES[level_index(wave)]
}

pub(crate) fn boss_skin(wave: i32) -> u8 {
    [SKIN_VEYRAN, SKIN_GUNNER, SKIN_VATBRUTE][level_index(wave)]
}

pub(crate) type HostileSpawn = (u8, u8, f32, f32);

const HOSTILES_UPPER: &[HostileSpawn] = &[
    (EK_HUSK, SKIN_RIFLEMAN, 7.5, 14.5),
    (EK_HUSK, SKIN_BREACHER, 14.5, 15.5),
    (EK_HUSK, SKIN_SUBJECT, 20.5, 13.5),
    (EK_BRUTE, SKIN_HAZMAT, 25.5, 17.5),
    (EK_HUSK, SKIN_RIFLEMAN, 22.5, 9.5),
    (EK_BRUTE, SKIN_GUNNER, 26.5, 3.5),
    (EK_WRAITH, SKIN_MARKSMAN, 19.5, 3.5),
    (EK_HUSK, SKIN_SUBJECT, 32.5, 15.5),
    (EK_BRUTE, SKIN_LOADER, 37.5, 13.5),
    (EK_WRAITH, SKIN_HORNET, 36.5, 21.5),
    (EK_BRUTE, SKIN_VATBRUTE, 41.5, 25.5),
    (EK_WRAITH, SKIN_HOUND, 38.5, 26.5),
    (EK_WRAITH, SKIN_SPITTER, 22.5, 26.5),
    (EK_BRUTE, SKIN_HAZMAT, 25.5, 25.5),
];
const HOSTILES_FOUNDRY: &[HostileSpawn] = &[
    (EK_HUSK, SKIN_HAZMAT, 8.5, 6.5),
    (EK_WRAITH, SKIN_HORNET, 16.5, 7.5),
    (EK_BRUTE, SKIN_LOADER, 22.5, 5.5),
    (EK_HUSK, SKIN_GUNNER, 33.5, 7.5),
    (EK_WRAITH, SKIN_MARKSMAN, 43.5, 5.5),
    (EK_HUSK, SKIN_RIFLEMAN, 21.5, 18.5),
    (EK_BRUTE, SKIN_HAZMAT, 17.5, 25.5),
    (EK_WRAITH, SKIN_HORNET, 25.5, 25.5),
    (EK_HUSK, SKIN_BREACHER, 32.5, 19.5),
    (EK_BRUTE, SKIN_LOADER, 37.5, 21.5),
    (EK_WRAITH, SKIN_HORNET, 43.5, 20.5),
    (EK_HUSK, SKIN_GUNNER, 41.5, 27.5),
];
const HOSTILES_BIOFORGE: &[HostileSpawn] = &[
    (EK_HUSK, SKIN_SUBJECT, 8.5, 14.5),
    (EK_BRUTE, SKIN_VATBRUTE, 16.5, 11.5),
    (EK_WRAITH, SKIN_SPITTER, 21.5, 9.5),
    (EK_HUSK, SKIN_HOUND, 27.5, 11.5),
    (EK_WRAITH, SKIN_SPITTER, 17.5, 20.5),
    (EK_BRUTE, SKIN_VATBRUTE, 26.5, 21.5),
    (EK_HUSK, SKIN_SUBJECT, 34.5, 6.5),
    (EK_WRAITH, SKIN_HOUND, 42.5, 6.5),
    (EK_WRAITH, SKIN_SPITTER, 34.5, 25.5),
    (EK_BRUTE, SKIN_VATBRUTE, 42.5, 25.5),
    (EK_HUSK, SKIN_HOUND, 34.5, 14.5),
    (EK_WRAITH, SKIN_SPITTER, 43.5, 18.5),
];

pub(crate) fn hostiles(wave: i32) -> &'static [HostileSpawn] {
    [HOSTILES_UPPER, HOSTILES_FOUNDRY, HOSTILES_BIOFORGE][level_index(wave)]
}

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

const FOUNDRY_AMBUSHES: &[AmbushDef] = &[
    AmbushDef {
        zone: (18.0, 3.0, 25.0, 10.5),
        group: &[(EK_WRAITH, 24.5, 11.5), (EK_HUSK, 18.5, 11.5)],
    },
    AmbushDef {
        zone: (31.0, 18.0, 44.0, 27.5),
        group: &[(EK_MARTYR, 34.5, 25.5), (EK_WRAITH, 43.5, 19.5)],
    },
];
const BIOFORGE_AMBUSHES: &[AmbushDef] = &[
    AmbushDef {
        zone: (15.0, 9.0, 30.0, 14.0),
        group: &[(EK_BRUTE, 18.5, 21.5), (EK_WRAITH, 27.5, 20.5)],
    },
    AmbushDef {
        zone: (32.0, 4.0, 45.0, 10.0),
        group: &[(EK_HUSK, 34.5, 8.5), (EK_WRAITH, 43.5, 8.5)],
    },
];

fn ambush_defs(wave: i32) -> &'static [AmbushDef] {
    [AMBUSH_DEFS, FOUNDRY_AMBUSHES, BIOFORGE_AMBUSHES][level_index(wave)]
}

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

    // Vault entrance: doored passage leading to the physical override station. Auto-opens on approach.
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

    mark_override(e, OVERRIDES[0]);
}

/// Long production lanes around a split foundry floor. Cross-connections make
/// combat loop around machinery instead of repeating the hub layout.
fn build_foundry(e: &mut Engine) {
    e.map.fill(1);
    e.floor.fill(0);
    e.decal.fill(0);
    e.room(1, 3, 10, 8, 7, 1);
    e.room(14, 2, 13, 11, 6, 1);
    e.room(30, 2, 16, 11, 4, 1);
    e.room(14, 17, 13, 12, 7, 0);
    e.room(30, 17, 16, 12, 4, 1);
    e.hall_h(10, 14, 6, 12);
    e.hall_h(26, 30, 7, 28);
    e.hall_v(20, 12, 17, 14);
    e.hall_v(37, 12, 17, 14);
    e.hall_h(26, 30, 23, 28);
    for (x, y) in [
        (18, 5),
        (23, 5),
        (18, 9),
        (23, 9),
        (34, 5),
        (41, 5),
        (34, 10),
        (41, 10),
        (18, 20),
        (23, 25),
        (34, 20),
        (41, 24),
    ] {
        e.pillar(x, y, if x < 28 { 6 } else { 4 });
    }
    for (x, y) in [(4, 3), (5, 3), (31, 28), (32, 28)] {
        e.set_cell(x, y, 9);
    }
    for (x, y) in [(4, 2), (5, 2), (31, 29), (32, 29)] {
        e.set_cell(x, y, 0);
    }
    e.mix_edge(1, 3, 10, 8, &[7, 1, 7, 4]);
    e.mix_edge(14, 2, 13, 11, &[6, 4, 7, 6]);
    e.mix_edge(30, 2, 16, 11, &[4, 6, 1, 7]);
    e.mix_edge(14, 17, 13, 12, &[7, 6, 4, 7]);
    e.mix_edge(30, 17, 16, 12, &[4, 7, 6, 4]);
    mark_override(e, OVERRIDES[1]);
}

/// Circular bio-research route with two specimen wings and a separate east
/// containment arena. Its ring offers flanking routes absent from the foundry.
fn build_bioforge(e: &mut Engine) {
    e.map.fill(1);
    e.floor.fill(0);
    e.decal.fill(0);
    e.room(1, 12, 10, 8, 3, 0);
    e.room(14, 8, 18, 16, 2, 0);
    e.room(32, 3, 14, 9, 3, 0);
    e.room(32, 20, 14, 9, 2, 0);
    e.room(32, 12, 14, 8, 5, 1);
    e.hall_h(10, 14, 15, 12);
    e.hall_h(31, 32, 10, 30);
    e.hall_h(31, 32, 21, 30);
    e.hall_h(31, 32, 15, 30);
    // Central isolation block creates a true loop through north/south labs.
    for y in 13..19 {
        for x in 20..26 {
            e.set_cell(x, y, 2);
        }
    }
    for (x, y) in [
        (17, 10),
        (28, 10),
        (17, 21),
        (28, 21),
        (35, 5),
        (42, 9),
        (35, 22),
        (42, 26),
        (36, 14),
        (41, 17),
    ] {
        e.pillar(x, y, if y < 12 || y > 20 { 3 } else { 5 });
    }
    for (x, y) in [(14, 13), (14, 14), (45, 15), (45, 16)] {
        e.set_cell(x, y, 9);
    }
    for (x, y) in [(13, 13), (13, 14), (46, 15), (46, 16)] {
        e.set_cell(x, y, 0);
    }
    e.mix_edge(1, 12, 10, 8, &[3, 2, 3, 4]);
    e.mix_edge(14, 8, 18, 16, &[2, 3, 5, 2]);
    e.mix_edge(32, 3, 14, 9, &[3, 2, 6, 3]);
    e.mix_edge(32, 20, 14, 9, &[2, 3, 5, 2]);
    e.mix_edge(32, 12, 14, 8, &[5, 2, 5, 3]);
    mark_override(e, OVERRIDES[2]);
}

fn mark_override(e: &mut Engine, point: (f32, f32)) {
    let cx = point.0.floor() as i32;
    let cy = point.1.floor() as i32;
    for y in cy - 1..=cy + 1 {
        for x in cx - 1..=cx + 1 {
            if e.cell(x, y) == 0 {
                e.floor[y as usize * MAP_W + x as usize] = 2;
            }
        }
    }
}

pub(crate) fn build_level(e: &mut Engine) {
    match level_index(e.wave) {
        1 => build_foundry(e),
        2 => build_bioforge(e),
        _ => build_hub_spoke(e),
    }
}

/// Props, pickups and the opening hostile cast. Zone by zone so balance
/// passes read like a playlist: hangar calm, lanes warm, zones hot.
pub(crate) fn place_hub_spoke(e: &mut Engine) {
    reset_level_entities(e);
    // Hangar: safe start, one medkit, crates to strafe around.
    e.spawn(EK_MED, 3.5, 17.5);
    e.spawn(EK_LAMP, 3.5, 13.5);
    e.spawn(EK_LAMP, 8.5, 13.5);
    e.spawn(EK_CRATE, 6.5, 17.5);
    e.spawn(EK_CRATE, 9.5, 16.5);
    e.spawn(EK_BARREL, 5.5, 13.5);
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
    // Vault: stocked for the override breach, barrels punish clustering.
    e.spawn(EK_MED, 43.5, 26.5);
    e.spawn(EK_AMMO, 39.5, 22.5);
    e.spawn(EK_BARREL, 35.5, 26.5);
    e.spawn(EK_BARREL, 42.5, 21.5);
    e.spawn(EK_LAMP, 39.5, 20.5);
    e.spawn(EK_CHAIN, 38.5, 24.5);
    // South lane + flesh pit.
    e.spawn(EK_CRATE, 22.5, 22.5);
    e.spawn(EK_GUN5, 22.5, 26.5);
    e.spawn(EK_BARREL, 18.5, 25.5);
    e.spawn(EK_BARREL, 20.5, 27.5);
    e.spawn(EK_MED, 26.5, 24.5);
    e.spawn(EK_AMMO, 19.5, 27.5);
    e.spawn(EK_LAMP, 22.5, 24.5);
    e.spawn(EK_BARREL, 26.5, 25.5);
    e.spawn(EK_ARMOR, 5.5, 20.5);
    e.spawn(EK_GUN3, 30.5, 12.5);
    e.spawn(EK_AMMO, 31.5, 13.5);
    e.spawn(EK_ARMOR, 31.5, 24.5);
    e.spawn(EK_MED, 32.5, 25.5);
    e.spawn(EK_GUN6, 41.5, 24.5);
    e.spawn(EK_GUN7, 4.5, 20.5);
}

fn reset_level_entities(e: &mut Engine) {
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
    e.ambush = ambush_defs(e.wave)
        .iter()
        .map(|d| AmbushTrigger {
            x0: d.zone.0,
            y0: d.zone.1,
            x1: d.zone.2,
            y1: d.zone.3,
            fired: false,
        })
        .collect();
}

fn place_foundry(e: &mut Engine) {
    reset_level_entities(e);
    for &(kind, x, y) in &[
        (EK_MED, 4.5, 8.5),
        (EK_ARMOR, 5.5, 2.5),
        (EK_CRATE, 8.5, 5.5),
        (EK_BARREL, 9.5, 8.5),
        (EK_AMMO, 16.5, 4.5),
        (EK_LAMP, 20.5, 3.5),
        (EK_CRATE, 24.5, 9.5),
        (EK_BARREL, 18.5, 9.5),
        (EK_MED, 32.5, 4.5),
        (EK_AMMO, 43.5, 10.5),
        (EK_LAMP, 37.5, 3.5),
        (EK_BARREL, 40.5, 9.5),
        (EK_ARMOR, 16.5, 27.5),
        (EK_AMMO, 24.5, 18.5),
        (EK_CRATE, 20.5, 23.5),
        (EK_BARREL, 24.5, 26.5),
        (EK_MED, 32.5, 27.5),
        (EK_AMMO, 43.5, 18.5),
        (EK_LAMP, 37.5, 27.5),
        (EK_BARREL, 34.5, 23.5),
    ] {
        e.spawn(kind, x, y);
    }
}

fn place_bioforge(e: &mut Engine) {
    reset_level_entities(e);
    for &(kind, x, y) in &[
        (EK_MED, 4.5, 18.5),
        (EK_ARMOR, 8.5, 13.5),
        (EK_CRATE, 9.5, 17.5),
        (EK_AMMO, 16.5, 15.5),
        (EK_LAMP, 18.5, 9.5),
        (EK_BARREL, 18.5, 22.5),
        (EK_MED, 29.5, 9.5),
        (EK_AMMO, 29.5, 22.5),
        (EK_CRATE, 28.5, 15.5),
        (EK_ARMOR, 33.5, 4.5),
        (EK_LAMP, 44.5, 10.5),
        (EK_BARREL, 38.5, 8.5),
        (EK_MED, 33.5, 27.5),
        (EK_AMMO, 44.5, 21.5),
        (EK_BARREL, 38.5, 24.5),
        (EK_AMMO, 34.5, 18.5),
        (EK_LAMP, 44.5, 13.5),
        (EK_CRATE, 37.5, 17.5),
    ] {
        e.spawn(kind, x, y);
    }
}

pub(crate) fn place_level(e: &mut Engine) {
    match level_index(e.wave) {
        1 => place_foundry(e),
        2 => place_bioforge(e),
        _ => place_hub_spoke(e),
    }
    let (x, y) = override_point(e.wave);
    let skin = [SKIN_CONSOLE_UPPER, SKIN_CONSOLE_FOUNDRY, SKIN_CONSOLE_BIOFORGE][level_index(e.wave)];
    e.spawn_with_skin(EK_OVERRIDE_CONSOLE, skin, x, y);
}

/// Fire each ambush once when the player crosses its zone. Called from
/// `tick` while the run is live. The wall-rumble cue reuses `EV_DOOR`
/// (no HUD-ABI change for a new event bit).
pub(crate) fn check_ambushes(e: &mut Engine) {
    use crate::events::EV_DOOR;
    for i in 0..e.ambush.len() {
        let group = ambush_defs(e.wave).get(i).map(|d| d.group).unwrap_or(&[]);
        let (fired, inside) = {
            let t = match e.ambush.get(i) {
                Some(t) => t,
                None => continue,
            };
            (
                t.fired,
                e.px >= t.x0 && e.px <= t.x1 && e.py >= t.y0 && e.py <= t.y1,
            )
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
    e.ents
        .iter()
        .filter(|x| x.hp > 0 && is_hostile_kind(x.kind))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flood fill from the player start. Doors (8) and secret doors (9)
    /// count as passable: the former auto-open on approach, the latter
    /// open with USE, so both are traversable in a real run.
    fn reachable_cells(e: &Engine) -> Vec<bool> {
        let mut seen = vec![false; MAP_CELLS];
        let start = player_start(e.wave);
        let mut stack = vec![(start.0.floor() as i32, start.1.floor() as i32)];
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
        // Lanes, zone centers, weapon pickups, override, secrets, ambush pockets.
        for (x, y, what) in [
            (22.5, 13.5, "plaza center"),
            (13.5, 15.5, "west lane"),
            (22.5, 9.5, "north lane"),
            (31.5, 15.5, "east lane"),
            (22.5, 22.5, "south lane"),
            (22.5, 3.5, "Scattergun"),
            (37.5, 12.5, "Lance"),
            (22.5, 26.5, "Pyre"),
            (41.5, 24.5, "AX-12"),
            (4.5, 20.5, "M91"),
            (41.5, 24.5, "override arena"),
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

    #[test]
    fn all_sector_layouts_are_distinct_and_objectives_are_reachable() {
        let mut e = Engine::new(160, 100);
        let mut layouts = Vec::new();
        for wave in 1..=LEVEL_COUNT as i32 {
            e.wave = wave;
            build_level(&mut e);
            let seen = reachable_cells(&e);
            let objective = override_point(wave);
            assert_reachable(&seen, objective.0, objective.1, "override station");
            for &(x, y) in boss_spots(wave) {
                assert_reachable(&seen, x, y, "boss arena");
            }
            for &(_, _, x, y) in hostiles(wave) {
                assert_reachable(&seen, x, y, "hostile spawn");
            }
            let idx = objective.1.floor() as usize * MAP_W + objective.0.floor() as usize;
            assert_eq!(e.floor[idx], 2, "override station needs its floor beacon");
            layouts.push(e.map.to_vec());
        }
        assert_ne!(layouts[0], layouts[1]);
        assert_ne!(layouts[1], layouts[2]);
        assert_ne!(layouts[0], layouts[2]);
    }

    #[test]
    fn each_sector_has_a_distinct_boss_and_enemy_cast() {
        let skins: Vec<u8> = (1..=3).map(boss_skin).collect();
        assert_eq!(skins, vec![SKIN_VEYRAN, SKIN_GUNNER, SKIN_VATBRUTE]);
        assert!(hostiles(1).len() > 10 && hostiles(2).len() > 10 && hostiles(3).len() > 10);
        assert_ne!(hostiles(1), hostiles(2));
        assert_ne!(hostiles(2), hostiles(3));
    }

    #[test]
    fn each_sector_places_one_visible_console_away_from_its_boss() {
        let mut e = Engine::new(160, 100);
        for wave in 1..=3 {
            e.wave = wave;
            build_level(&mut e);
            place_level(&mut e);
            let consoles: Vec<_> = e.ents.iter().filter(|ent| ent.kind == EK_OVERRIDE_CONSOLE).collect();
            assert_eq!(consoles.len(), 1);
            assert_eq!((consoles[0].x, consoles[0].y), override_point(wave));
            assert_eq!(consoles[0].skin, [SKIN_CONSOLE_UPPER, SKIN_CONSOLE_FOUNDRY, SKIN_CONSOLE_BIOFORGE][level_index(wave)]);
            assert!(boss_spots(wave).iter().all(|spot| *spot != override_point(wave)));
        }
    }
}
