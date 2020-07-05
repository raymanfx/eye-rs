/// Integer control knob
pub struct Integer {
    /// Valid value range (inclusive on both ends)
    pub range: (i64, i64),
    /// Valid range step size
    pub step: u64,
    /// Default value
    pub default: i64,
}
/// Device control menu item
pub enum MenuItem {
    /// String representation, use this as fallback
    String(String),
    /// Integer representation
    Integer(i64),
}

/// Device control representation
pub enum Representation {
    /* Unknown */
    Unknown,

    /* Stateless controls */
    Button,

    /* Single value controls */
    Boolean,
    Integer(Integer),
    String,

    /* Multi value controls */
    Bitmask,
    Menu(Vec<MenuItem>),
}
