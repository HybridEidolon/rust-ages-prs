//! PRS flavor policies. Applications usually expect and produce
//! particular variations on PRS.

use std::u8::MAX as U8MAX;

/// Flavor of PRS compression used. Varies with target game
pub trait Flavor {
    const MIN_LONG_COPY_LENGTH: u16;
    const MAX_COPY_LENGTH: u16 = U8MAX as u16 + Self::MIN_LONG_COPY_LENGTH;
}

/// PRS used for games in the Dreamcast and Saturn era.
///
/// - Phantasy Star Online
/// - Sonic Adventure
/// - NiGHTS Into Dreams
/// - likely others
pub enum Legacy {}

impl Flavor for Legacy {
    const MIN_LONG_COPY_LENGTH: u16 = 1;
}

/// Modern PRS used in games made after the Dreamcast.
///
/// - Phantasy Star Universe
/// - Phantasy Star Online 2
pub enum Modern {}

impl Flavor for Modern {
    const MIN_LONG_COPY_LENGTH: u16 = 10;
}
