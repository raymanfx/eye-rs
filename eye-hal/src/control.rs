use bitflags::bitflags;

#[derive(Debug, Clone)]
/// Device control
pub struct Descriptor {
    /// Unique identifier
    pub id: u32,
    /// Name
    pub name: String,
    /// State type
    pub typ: Type,
    /// State flags
    pub flags: Flags,
}

impl Descriptor {
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
/// Device control type
pub enum Type {
    /// Stateless controls
    Stateless,
    /// On/Off switch
    Boolean,
    /// Numerical control
    Number {
        /// Valid value range (inclusive on both ends)
        range: (f64, f64),
        /// Valid range step size
        step: f32,
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
    /// String value
    String(String),
    /// Numerical value
    Number(f64),
}

bitflags! {
    /// Control state flags
   #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
/// Device control state
pub enum State {
    /* Stateless controls */
    None,

    /* Single value controls */
    String(String),
    Boolean(bool),
    Number(f64),
}
