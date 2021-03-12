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
        self.flags & Flags::READ == Flags::READ
    }

    /// Returns true if the control value can be written
    pub fn writable(&self) -> bool {
        self.flags & Flags::WRITE == Flags::WRITE
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
        /// Value can be read
        const READ                  = 0x001;
        /// Value can be written
        const WRITE                 = 0x002;
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
