use messages::node::{Node, Node::TemperatureBoard, Node::PressureBoard, Node::StrainBoard};

pub enum Feature {
    Temperature,
    Pressure,
    Strain,
}

pub const FEATURE: Feature = if cfg!(feature = "temperature") {
    Feature::Temperature
} else if cfg!(feature = "pressure") {
    Feature::Pressure
} else if cfg!(feature = "strain") {
    Feature::Strain
} else {
    Feature::Pressure // dummy return the code will not compile in this case. 
};

pub const ADC2_RST_PIN_PORT: char = match FEATURE {
    Feature::Temperature => 'E',
    Feature::Pressure => 'D',
    Feature::Strain => 'B',
};

pub const ADC2_RST_PIN_ID: u8 = match FEATURE {
    Feature::Temperature => 0,
    Feature::Pressure => 1,
    Feature::Strain => 9,
};

pub static COM_ID: Node = match FEATURE {
    Feature::Temperature => TemperatureBoard,
    Feature::Pressure => PressureBoard,
    Feature::Strain => StrainBoard,
};
// #[cfg(not(any(feature = "pressure", feature = "temperature", feature = "strain")))]
// compile_error!("You must enable exactly one of the features: 'pressure', 'temperature', or 'strain'.");

// use messages::node::{Node, Node::TemperatureBoard, Node::PressureBoard, Node::StrainBoard};
// use stm32h7xx_hal::gpio::{Output, Pin, PushPull};


// pub const ADC2_RST_PIN_PORT: char = if cfg!(feature = "temperature") {
//     'E'
// } else if cfg!(feature = "pressure") {
//     'D'
// } else if cfg!(feature = "strain") {
//     'B'
// } else {
//     'D'
// }; 

// pub const ADC2_RST_PIN_ID: u8 = if cfg!(feature = "temperature") {
//     0
// } else if cfg!(feature = "pressure") {
//     1
// } else {
//     9
// };

// #[cfg(feature = "temperature")]
// pub static COM_ID: Node = TemperatureBoard;

// #[cfg(feature = "pressure")]
// pub static COM_ID: Node = PressureBoard;

// #[cfg(feature = "strain")]
// pub static COM_ID: Node = StrainBoard;