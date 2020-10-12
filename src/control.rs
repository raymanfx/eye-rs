use bitflags::bitflags;

#[derive(Debug, Clone)]
/// Device control
pub struct Control {
    /// Unique identifier
    pub id: u32,
    /// Name
    pub name: String,
    /// Representation for UIs
    pub repr: Representation,
    /// State flags
    pub flags: Flags,
}

impl Control {
    /// Returns true if the control value can be read
    pub fn readable(&self) -> bool {
        !(self.flags & Flags::WRITE_ONLY == Flags::WRITE_ONLY
            || self.flags & Flags::INACTIVE == Flags::INACTIVE)
    }

    /// Returns true if the control value can be written
    pub fn writable(&self) -> bool {
        !(self.flags & Flags::READ_ONLY == Flags::READ_ONLY
            || self.flags & Flags::GRABBED == Flags::GRABBED)
    }
}

#[derive(Debug, Clone)]
/// Device control representation
pub enum Representation {
    /// Unknown
    Unknown,
    /// Stateless controls
    Button,
    /// On/Off switch
    Boolean,
    /// Integer control
    Integer {
        /// Valid value range (inclusive on both ends)
        range: (i64, i64),
        /// Valid range step size
        step: u64,
        /// Default value
        default: i64,
    },
    /// String control
    String,
    /// Bit field
    Bitmask,
    /// Menu containing an arbitrary number of items
    Menu(Vec<MenuItem>),
}

#[derive(Debug, Clone)]
/// Device control menu item
pub enum MenuItem {
    /// String representation, use this as fallback
    String(String),
    /// Integer representation
    Integer(i64),
}

bitflags! {
    /// Control state flags
    pub struct Flags: u32 {
        /// No flags are set
        const NONE                  = 0x000;
        /// Permanently read-only
        const READ_ONLY             = 0x001;
        /// Permanently write-only
        const WRITE_ONLY            = 0x002;
        /// Grabbed by another process, temporarily read-only
        const GRABBED               = 0x004;
        /// Not applicable in the current context
        const INACTIVE              = 0x008;
    }
}

#[derive(Debug, Clone)]
/// Device control value representation
pub enum Value {
    /* Stateless controls */
    None,

    /* Single value controls */
    String(String),
    Boolean(bool),
    Integer(i64),
}
