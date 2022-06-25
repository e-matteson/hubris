// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

use userlib::*;

struct Client {
    id: TaskId,
    notification: u32,
}

#[export_name = "main"]
pub fn main() -> ! {
    const TIMER_NOTIFICATION: u32 = 1;
    const INTERVAL: u64 = 250;
    const SUBSCRIBE_OP: u32 = 5;
    const GET_OP: u32 = 6;

    let mut client: Option<Client> = None;

    let mut msg_in = [0; 4];

    let mut info_to_send: u8 = 0;

    let mut dl = INTERVAL;
    sys_set_timer(Some(dl), TIMER_NOTIFICATION);
    loop {
        let msginfo = sys_recv_open(&mut msg_in, TIMER_NOTIFICATION);

        if msginfo.sender == TaskId::KERNEL {
            // This is a notification message. We've only got one notification
            // enabled, so we know full well which it is without looking.
            dl += INTERVAL;
            sys_set_timer(Some(dl), TIMER_NOTIFICATION);
            if let Some(client) = &client {
                sys_post(client.id, client.notification);
                info_to_send = (info_to_send + 1) % 2;
            }
        } else {
            match msginfo.operation {
                SUBSCRIBE_OP => {
                    assert!(msginfo.message_len == 4);
                    let notification = u32::from_le_bytes(msg_in);
                    assert!(client.is_none());
                    client = Some(Client {
                        id: msginfo.sender,
                        notification,
                    });
                    sys_reply(msginfo.sender, 0, &[]);
                }
                GET_OP => {
                    assert!(msginfo.message_len == 0);
                    assert!(client.as_ref().unwrap().id == msginfo.sender);
                    sys_reply(msginfo.sender, 0, &[info_to_send]);
                }
                _ => panic!("unknown op"),
            }
        }
    }
}
