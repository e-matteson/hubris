// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

use stm32f4::stm32f407 as device;
use userlib::*;
use zerocopy::AsBytes;

task_slot!(RCC, rcc_driver);

struct Client {
    id: TaskId,
    notification: u32,
}

#[export_name = "main"]
pub fn main() -> ! {
    const TIMER_NOTIFICATION: u32 = 1;
    const EXTI_NOTIFICATION_MASK: u32 = 2;
    const INTERVAL_MS: u64 = 250;
    const SUBSCRIBE_OP: u32 = 5;
    const GET_OP: u32 = 6;

    turn_on_gpioa();

    let gpioa = unsafe { &*device::GPIOA::ptr() };

    // Pin A0 is the pushbutton on the f4 discovery board.
    // TODO this is the default state, so we don't actually need to modify registers...
    gpioa.moder.modify(|_, w| w.moder0().input());
    // gpioa.pupdr.modify(|_, w| w.pupdr0().no_pull());

    configure_ext_interrupts(EXTI_NOTIFICATION_MASK);

    let mut client: Option<Client> = None;

    let mut msg_in = [0; 4];

    let mut info_to_send: u8 = 0;

    let mut dl = INTERVAL_MS;
    sys_set_timer(Some(dl), TIMER_NOTIFICATION);
    loop {
        let msginfo = sys_recv_open(
            &mut msg_in,
            TIMER_NOTIFICATION | EXTI_NOTIFICATION_MASK,
        );

        if msginfo.sender == TaskId::KERNEL {
            if (msginfo.operation & TIMER_NOTIFICATION) != 0 {
                dl += INTERVAL_MS;
                sys_set_timer(Some(dl), TIMER_NOTIFICATION);
                // // Read gpio A0
                // let button_state: bool = (gpioa.idr.read().bits() & 0x01) != 0;
                // if button_state {
                //     info_to_send = (info_to_send + 1) % 2;
                // }

                // let button_state: bool = (gpioa.idr.read().bits() & 0x01) != 0;
                // if button_state {
                //     if let Some(client) = &client {
                //         sys_post(client.id, client.notification);
                //     }
                // }

                // Manually generate interrupt request
                let exti = unsafe { &*device::EXTI::ptr() };
                exti.swier.modify(|_, w| w.swier0().set_bit());
            }
            if (msginfo.operation & EXTI_NOTIFICATION_MASK) != 0 {
                if let Some(client) = &client {
                    info_to_send = (info_to_send + 1) % 2;
                    sys_post(client.id, client.notification);
                }
                // Unmask the interrupt so it can fire again
                sys_irq_control(EXTI_NOTIFICATION_MASK, true);
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

fn configure_ext_interrupts(exti_notification_mask: u32) {
    // TODO might need to use SYSCFG_EXTICRn to pick which port's pin n should
    // be the input source for the EXTIn interrupt. The default is port A, which
    // is conveniently what the pushbutton is on.

    let exti = unsafe { &*device::EXTI::ptr() };

    // Enable/mask exti0
    exti.imr.modify(|_, w| w.mr0().masked());
    // Trigger on rising edge
    exti.rtsr.modify(|_, w| w.tr0().enabled());

    sys_irq_control(exti_notification_mask, true);
}

fn turn_on_gpioa() {
    // TODO how does this function work?

    let rcc_driver = RCC.get_task_id();

    const ENABLE_CLOCK: u16 = 1;

    let pnum: u32 = 0; // see bits in AHB1ENR

    let (code, _) = userlib::sys_send(
        rcc_driver,
        ENABLE_CLOCK,
        pnum.as_bytes(),
        &mut [],
        &[],
    );
    assert_eq!(code, 0);

    const LEAVE_RESET: u16 = 4;
    let (code, _) = userlib::sys_send(
        rcc_driver,
        LEAVE_RESET,
        pnum.as_bytes(),
        &mut [],
        &[],
    );
    assert_eq!(code, 0);
}
