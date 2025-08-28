//! Simplified math operations module

pub mod abs;
pub mod ceiling;
pub mod exp;
pub mod floor;
pub mod ln;
pub mod log;
pub mod power;
pub mod precision;
pub mod round;
pub mod sqrt;
pub mod truncate;

// Arithmetic operations
pub mod add;
pub mod divide;
pub mod modulo;
pub mod multiply;
pub mod subtract;

pub use abs::SimpleAbsFunction;
pub use ceiling::SimpleCeilingFunction;
pub use exp::SimpleExpFunction;
pub use floor::SimpleFloorFunction;
pub use ln::SimpleLnFunction;
pub use log::SimpleLogFunction;
pub use power::SimplePowerFunction;
pub use precision::SimplePrecisionFunction;
pub use round::SimpleRoundFunction;
pub use sqrt::SimpleSqrtFunction;
pub use truncate::SimpleTruncateFunction;

// Arithmetic operations
pub use add::SimpleAddFunction;
pub use divide::SimpleDivideFunction;
pub use modulo::SimpleModuloFunction;
pub use multiply::SimpleMultiplyFunction;
pub use subtract::SimpleSubtractFunction;
