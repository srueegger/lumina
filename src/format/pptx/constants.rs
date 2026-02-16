/// 1 point = 12700 EMU (English Metric Units)
pub const EMU_PER_PT: f64 = 12700.0;

/// Convert EMU to points
pub fn emu_to_pt(emu: i64) -> f64 {
    emu as f64 / EMU_PER_PT
}

/// Convert half-points (used for font sizes) to points
pub fn half_pt_to_pt(half_pt: f64) -> f64 {
    half_pt / 100.0
}
