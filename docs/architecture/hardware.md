# Hardware Abstraction

## Purpose

The hardware abstraction layer (HAL) separates application and runtime code from board-, device-, and driver-specific details.

## Conceptual boundary

```text
Application / Runtime
        │
        ▼
      NROS HAL
        │
        ├── Sensors
        ├── Actuators
        ├── Timing
        ├── Memory / buffers
        └── Device control
        │
        ▼
Drivers / OS / Hardware
```

## Design goals

- stable interfaces across hardware implementations;
- explicit capability and ownership boundaries;
- predictable error handling;
- support for real-time constraints where required;
- a clear distinction between simulated devices and physical devices.

## DMA and zero-copy

A HAL abstraction for a DMA buffer is not evidence that physical DMA or DMA-BUF integration exists. Documentation must identify simulated, scaffolded, and hardware-backed implementations separately.

## Validation

Hardware claims require evidence from the relevant device and platform. Unit tests and simulations can validate software behavior but do not substitute for hardware validation.
