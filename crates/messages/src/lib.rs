#![no_std]

pub mod sensor {
    use crate::common;

    pub mod sbg {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.sbg.rs"));
    }

    pub mod gps {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.gps.rs"));
    }

    pub mod madgwick {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.madgwick.rs"));
    }

    pub mod iim20670 {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.iim20670.rs"));
    }

    pub mod ms5611 {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.ms5611.rs"));
    }

    pub mod argus {
        include!(concat!(env!("OUT_DIR"), "/messages_prost.argus.rs")); 
    }
}


pub mod common {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.common.rs"));
}

pub mod argus_state {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.argus_state.rs"));
}

pub mod phoenix_state {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.phoenix_state.rs"));
}

pub mod log {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.log.rs"));
}

pub mod command {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.command.rs"));
}

pub mod radio {
    include!(concat!(env!("OUT_DIR"), "/messages_prost.radio.rs"));
}

// Top-level shims to satisfy generated type paths referenced by `radio` oneof
// while preserving the existing `sensor::*` public API.
pub mod sbg {
    pub use crate::sensor::sbg::*;
}

pub mod gps {
    pub use crate::sensor::gps::*;
}

pub mod madgwick {
    pub use crate::sensor::madgwick::*;
}

pub mod iim20670 {
    pub use crate::sensor::iim20670::*;
}

pub mod ms5611 {
    pub use crate::sensor::ms5611::*;
}

pub mod argus {
    pub use crate::sensor::argus::*;
}

pub use mavlink;
pub use prost;