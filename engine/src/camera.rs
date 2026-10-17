//! Responsive mouse look and presentation-only player interpolation.
use crate::*;

pub(crate) const PITCH_LIMIT: f32 = 0.85;
const LOOK_GAIN: f32 = 0.0062;
// Horizontal focal length is height / (2 * (0.72 / 1.6)); a pitch unit
// shifts the horizon by height * 0.9. Match their screen-space mouse gain.
const PITCH_GAIN: f32 = LOOK_GAIN / (2.0 * (0.72 / 1.6) * 0.9);

pub(crate) struct Camera {
    previous: [f32; 2],
    position: [f32; 2],
    active: bool,
}

impl Camera {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self { previous: [x, y], position: [x, y], active: false }
    }
}

impl Engine {
    /// Mouse deltas are displacement, independent of simulation/display rate.
    /// Apply before combat as well as rendering; no filtering or acceleration.
    pub(crate) fn look(&mut self, mx: f32, my: f32) {
        if self.state != 0 { return; }
        self.pa += finite(mx) * LOOK_GAIN;
        self.pitch = (self.pitch - finite(my) * PITCH_GAIN).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        self.hud.yaw = self.pa;
    }

    pub(crate) fn reset_camera(&mut self) {
        self.camera = Camera::new(self.px, self.py);
    }

    pub(crate) fn begin_camera_step(&mut self) {
        self.camera.previous = [self.px, self.py];
        self.camera.active = false;
    }

    /// Return two f32s (x, y). Rendering must not move the simulation player.
    pub(crate) fn prepare_camera(&mut self, alpha: f32) -> *const f32 {
        let previous = self.camera.previous;
        let distance = (self.px - previous[0]).hypot(self.py - previous[1]);
        // Teleports, spawn corrections and stopped games must never pan through walls.
        let a = if self.state != 0 || distance > 0.5 || !alpha.is_finite() { 1.0 }
            else { alpha.clamp(0.0, 1.0) };
        self.camera.position = [
            previous[0] + (self.px - previous[0]) * a,
            previous[1] + (self.py - previous[1]) * a,
        ];
        self.camera.active = true;
        self.camera.position.as_ptr()
    }

    pub(crate) fn view_position(&self) -> (f32, f32) {
        if self.camera.active { (self.camera.position[0], self.camera.position[1]) }
        else { (self.px, self.py) }
    }
}
