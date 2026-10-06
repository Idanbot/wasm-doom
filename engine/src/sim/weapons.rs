//! Weapon ownership and the mouse-wheel/number-key selection model.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    /// Ownership flag for slot `i + 1` of the `owned` array.
    #[inline]
    pub(crate) fn has_w(&self, i: usize) -> bool {
        self.owned.get(i).copied().unwrap_or(false)
    }

    /// Same flag in the 0/1 form the `Hud` wire struct carries.
    #[inline]
    pub(crate) fn owned_flag(&self, i: usize) -> i32 {
        self.has_w(i) as i32
    }

    /// Grants ownership of slot `i + 1`.
    #[inline]
    pub(crate) fn set_w(&mut self, i: usize) {
        if let Some(slot) = self.owned.get_mut(i) {
            *slot = true;
        }
    }

    pub(crate) fn owns_slot(&self, slot: usize) -> bool {
        if slot >= 19 { return slot < WEP_N && self.extra_weapons & (1 << (slot - 19)) != 0; }
        if slot == 0 { return true; }
        self.has_w(slot - 1)
    }

    pub(crate) fn select_weapon(&mut self, slot: usize) {
        if slot < WEP_N && self.owns_slot(slot) && slot != self.weapon as usize {
            self.weapon = slot as i32;
            self.reload_t = 0.0;
            self.pickup_t = 0.28;self.pickup_dur=0.28;
            self.cooldown=self.cooldown.min(0.12);
        }
    }

    /// QA-only full heal so long single-page smokes don't die mid-run.
    /// Keeps weapons, armor and position; revives and clears hostiles,
    /// in-flight projectiles and lingering fire so a converged crowd
    /// can't wedge the rest of the script.
    pub(crate) fn qa_heal(&mut self) {
        if !self.qa { return; }
        self.health = 100;
        self.pending_hostiles = 0;
        self.iframes = 1.5;
        self.state = 0;
        for ent in &mut self.ents {
            if is_hostile_kind(ent.kind)
                || matches!(ent.kind, EK_PROJ | EK_FIREPATCH | EK_FLAME)
            {
                ent.kind = EK_NONE;
                ent.hp = 0;
            }
        }
    }
}
