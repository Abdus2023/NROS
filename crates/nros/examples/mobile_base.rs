//! Mobile base example using full NROS API with macros — now compiles thanks to nros facade + nros-macros
//! Status: SCAFFOLDED-IMPLEMENTED per AUDIT.md — macros are passthrough now, real codegen future
//! This file demonstrates DESIGN.md §3.1 programming model and is included to make `nros init` generated code concept compile
//! Run: cargo check -p nros --example mobile_base

use nros::prelude::*;

#[nros::node]
struct MobileBase {
    #[subscribe(topic = "/cmd_vel")]
    cmd_vel: Subscriber<Twist>,

    #[publish(topic = "/odom")]
    odom_pub: Publisher<Odometry>,

    #[publish(topic = "/motor_commands")]
    motor_pub: Publisher<MotorCommand>,

    #[param(default = 1.0, min = 0.1, max = 10.0)]
    max_speed: f64,
}

impl MobileBase {
    #[callback(realtime = true, deadline_us = 1000)]
    fn on_cmd_vel(&mut self, msg: Twist) {
        // Would compute motor commands — here just placeholder
        let _ = msg;
    }

    #[callback(frequency = 50)]
    fn control_loop(&mut self) {
        // Control logic
    }
}

fn main() {
    nros::init();
    println!("MobileBase example with #[nros::node] — compiles thanks to proc-macro passthrough");
    println!("In real NROS, this would spin: nros::spin(node)");
    // let node = MobileBase { cmd_vel: ..., odom_pub: ..., motor_pub: ..., max_speed: 2.0 };
    // nros::spin(node);
}
