//! This crate provides basic functionnality to communicate with Herkulex DRS (both 0101 and 0201)
//! servomotors.
//! It is heavily based on the documentation published by Dongbu Robot which is available
//! [`here`](http://www.sgbotic.com/products/datasheets/robotics/herkulexeng.pdf).
//!
//! # Examples
//!
//! To reboot all the servomotors you can use this message :
//!
//! ```
//! # extern crate drs_0x01;
//! use drs_0x01::MessageBuilder;
//! // 0xFE is the broadcast ID
//! let message = MessageBuilder::new().id(0xFE).reboot().build();
//! ```
//!
//! Here is how to enable torque for the servomotor labelled 35 :
//!
//! ```
//! # extern crate drs_0x01;
//! use drs_0x01::MessageBuilder;
//! use drs_0x01::WritableRamAddr::TorqueControl;
//! let message = MessageBuilder::new_with_id(35).write_ram(TorqueControl(1)).build();
//! ```

#![no_std]
#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate arrayvec;

mod addr;
mod builder;
mod message;

pub use builder::{MessageBuilder, MessageBuilderCmd, MessageBuilderError,
                  MessageBuilderPositionIJOG, MessageBuilderPositionSJOG, MessageBuilderSpecial};

pub use addr::*;

pub use message::{JogColor, JogMode};
