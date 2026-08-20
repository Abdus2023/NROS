//! NROS procedural macros — SCAFFOLDED per AUDIT.md Pass 2 & 5
//! Status: SCAFFOLDED — provides #[nros::node] etc as no-op passthrough to allow generated projects to compile
//! Real implementation would: parse struct fields, generate Publisher/Subscriber wiring, param validation, lifecycle, QoS, etc.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item, ItemStruct, ItemFn, Attribute};

/// #[nros::node] or #[nros_macros::node] — marks struct as NROS node
/// Currently passthrough (SCAFFOLDED), real would generate lifecycle impl, parameter handling, etc.
#[proc_macro_attribute]
pub fn node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _attr = attr;
    // Parse as struct to ensure it's valid, then return unchanged (passthrough)
    let input = parse_macro_input!(item as ItemStruct);
    let output = quote! { #input };
    output.into()
}

/// #[subscribe(topic = "/cmd_vel", qos = Reliable)] — marks field as subscriber
/// SCAFFOLDED: currently no-op, real would generate subscription registration + validation
#[proc_macro_attribute]
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _attr = attr;
    // Allow field or struct — passthrough
    // For field attributes, syn may parse differently, so we just return item
    item
}

/// #[publish(topic = "/motor_commands")] — marks field as publisher
#[proc_macro_attribute]
pub fn publish(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[param(default = 1.0, min = 0.1, max = 10.0)] — parameter with validation
#[proc_macro_attribute]
pub fn param(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[service(name = "/reset_controller")] — service provider
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[callback(realtime = true, deadline_us = 1000)] — marks callback
#[proc_macro_attribute]
pub fn callback(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[time_sync(tolerance_ms = 5)] — time synchronized multi-subscriber
#[proc_macro_attribute]
pub fn time_sync(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[compute(prefer = "GPU")] / #[compute(device = "NPU:0")] — heterogeneous compute dispatch
#[proc_macro_attribute]
pub fn compute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[interrupt(priority = 255, latency_ns = 500)] — zero-latency ISR
#[proc_macro_attribute]
pub fn interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[distributed_node] — distributed computing node
#[proc_macro_attribute]
pub fn distributed_node(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[shared_state(consensus = "raft")] — shared state with consensus
#[proc_macro_attribute]
pub fn shared_state(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[task(distributed = true)] — distributed task
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[sim(model = "models/my_robot.urdf")] — simulation model
#[proc_macro_attribute]
pub fn sim(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

/// #[plugin] / #[plugin_impl] / #[algorithm] / etc for plugin system
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }

#[proc_macro_attribute]
pub fn plugin_impl(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }

#[proc_macro_attribute]
pub fn algorithm(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }

#[proc_macro_attribute]
pub fn algorithm_impl(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }

#[proc_macro_attribute]
pub fn telemetry(attr: TokenStream, item: TokenStream) -> TokenStream { let _ = attr; item }

// Additional helper to parse any item as-is for robustness
fn _parse_any(item: TokenStream) -> TokenStream {
    // Try to parse as Item, if fails return as-is
    item
}
