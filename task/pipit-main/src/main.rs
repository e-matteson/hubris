// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

use userlib::*;

task_slot!(USER_LEDS, user_leds);
task_slot!(PIPIT_MATRIX_SCANNER, pipit_matrix_scanner);

#[export_name = "main"]
pub fn main() -> ! {
    const MATRIX_NOTIFICATION: u32 = 2;
    const SUBSCRIBE_OP: u16 = 5;
    const GET_OP: u16 = 6;

    let user_leds = drv_user_leds_api::UserLeds::from(USER_LEDS.get_task_id());

    let (response, length) = sys_send(
        PIPIT_MATRIX_SCANNER.get_task_id(),
        SUBSCRIBE_OP,
        &MATRIX_NOTIFICATION.to_le_bytes(),
        &mut [],
        &[],
    );
    assert!(response == 0);
    assert!(length == 0);

    loop {
        let msginfo = sys_recv_open(&mut [], MATRIX_NOTIFICATION);
        if msginfo.sender == TaskId::KERNEL
            && msginfo.operation == MATRIX_NOTIFICATION
        {
            let mut msg_in = [0; 1];
            let (response, length) = sys_send(
                PIPIT_MATRIX_SCANNER.get_task_id(),
                GET_OP,
                &[],
                &mut msg_in,
                &[],
            );
            assert!(response == 0);
            assert!(length == 1);
            let led_index: usize = msg_in[0].into();

            user_leds.led_toggle(led_index).unwrap();
        }
    }
}
