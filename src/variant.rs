//! PRS variant policies. Applications usually expect and produce particular
//! variations on PRS.

use std::u8::MAX as U8MAX;

/// Variant of PRS compression used. Varies with target game.
///
/// This trait is sealed from implementation by downstream consumers, because
/// improper impls of this trait may result in panics in the implementation. If
/// you have a variant of PRS that is not supported here, please open an issue
/// on the issue tracker.
pub trait Variant: private::Sealed {
    #[doc(hidden)]
    const MIN_LONG_COPY_LENGTH: u16;
    #[doc(hidden)]
    const MAX_COPY_LENGTH: u16 = U8MAX as u16 + Self::MIN_LONG_COPY_LENGTH;
}

/// PRS Variant used in games in the Dreamcast and Saturn era.
///
/// - Phantasy Star Online
/// - Sonic Adventure
/// - NiGHTS Into Dreams
/// - likely others
pub enum Legacy {}

impl Variant for Legacy {
    #[doc(hidden)]
    const MIN_LONG_COPY_LENGTH: u16 = 1;
}

/// PRS Variant used in games made after the Dreamcast.
///
/// - Phantasy Star Universe
/// - Phantasy Star Online 2
pub enum Modern {}

impl Variant for Modern {
    #[doc(hidden)]
    const MIN_LONG_COPY_LENGTH: u16 = 10;
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Legacy {}
    impl Sealed for super::Modern {}
}
