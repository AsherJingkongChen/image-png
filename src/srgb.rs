use crate::{ScaledFloat, SourceChromaticities};

/// A value scaled as per specification, for the constants below: they are evaluated when the
/// crate is compiled, so one out of range stops the build instead of reaching a caller.
const fn scaled(val: u32) -> ScaledFloat {
    match ScaledFloat::from_scaled(val) {
        Ok(s) => s,
        Err(_) => panic!("a scaled float is a PNG four-byte unsigned integer"),
    }
}

// Values taken from https://www.w3.org/TR/2003/REC-PNG-20031110/#11sRGB
const GAMMA: ScaledFloat = scaled(45455);
const CHROMATICITIES: SourceChromaticities = SourceChromaticities {
    white: (scaled(31270), scaled(32900)),
    red: (scaled(64000), scaled(33000)),
    green: (scaled(30000), scaled(60000)),
    blue: (scaled(15000), scaled(6000)),
};

/// Get the gamma that should be substituted for images conforming to the sRGB color space.
pub fn substitute_gamma() -> ScaledFloat {
    GAMMA
}

/// Get the chromaticities that should be substituted for images conforming to the sRGB color space.
pub fn substitute_chromaticities() -> SourceChromaticities {
    CHROMATICITIES
}
