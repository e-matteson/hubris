// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Client API for the VPD task.

#![no_std]

use derive_idol_err::IdolError;
use drv_i2c_api::ResponseCode;
use userlib::*;

#[derive(Copy, Clone, Debug, FromPrimitive, Eq, PartialEq, IdolError)]
pub enum VpdError {
    InvalidDevice = 1,
    NotPresent = 2,
    DeviceError = 3,
    Unavailable = 4,
    DeviceTimeout = 5,
    DeviceOff = 6,
    BadAddress = 7,
    BadBuffer = 8,
    BadRead = 9,
    BadWrite = 10,
}

impl From<ResponseCode> for VpdError {
    fn from(code: ResponseCode) -> VpdError {
        match code {
            ResponseCode::NoDevice => VpdError::NotPresent,
            ResponseCode::NoRegister => VpdError::Unavailable,
            ResponseCode::BusLocked
            | ResponseCode::BusLockedMux
            | ResponseCode::ControllerLocked => VpdError::DeviceTimeout,
            _ => VpdError::DeviceError,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/client_stub.rs"));
