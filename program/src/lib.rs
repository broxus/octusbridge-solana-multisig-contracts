mod error;
mod instruction;
mod processor;
mod state;
mod utils;

pub use self::error::*;
pub use self::instruction::*;
pub use self::processor::*;
pub use self::state::*;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
extern crate wasm_bindgen;

#[cfg(feature = "wasm")]
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm;

solana_program::declare_id!("9RVX1zPChY78ZNwAmMgtdwfupGrbBz1vJRCqPFnTzrMW");
