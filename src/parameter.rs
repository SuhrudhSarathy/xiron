pub const RESOLUTION: f32 = 0.025;
pub const DT: f32 = 1.0 / 60.0;
pub const XLIMS: (f32, f32) = (-15.0, 15.0);
pub const YLIMS: (f32, f32) = (-15.0, 15.0);
pub const WIDTH: f32 = (XLIMS.1 - XLIMS.0) / RESOLUTION;
pub const HEIGHT: f32 = (YLIMS.1 - YLIMS.0) / RESOLUTION;

// TODO: xlims and ylims should scale accrding to the height and width of the Window
// and not the other way around?
