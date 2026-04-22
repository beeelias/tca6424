# tca6424

`no_std` Rust driver for the [TCA6424A](https://www.ti.com/product/TCA6424A) 24-bit I2C GPIO expander.

Supports both synchronous and asynchronous I2C via feature flags, built on [`embedded-hal`](https://docs.rs/embedded-hal) v1.

## Features

| Feature | Description |
|---------|-------------|
| *(default)* | Synchronous API using `embedded-hal::i2c::I2c` |
| `async` | Asynchronous API using `embedded-hal-async::i2c::I2c` |
| `defmt` | `defmt::Format` derives and structured logging |
| `log` | `log` crate logging |

## Usage

```toml
# Synchronous (default)
[dependencies]
tca6424 = "0.1"

# Async (e.g. Embassy on STM32)
[dependencies]
tca6424 = { version = "0.1", default-features = false, features = ["async", "defmt"] }
```

```rust
use tca6424::{Tca6424a, Port};

let mut expander = Tca6424a::new(i2c, 0x22);

// Initialize: config = port directions (0=output, 1=input), output = initial values
expander.init(
    [0x00, 0xF0, 0xFF], // P0: all outputs, P1: lower outputs/upper inputs, P2: all inputs
    [0x00, 0x00, 0x00], // all outputs start low
).await?; // drop .await for sync

// Set specific bits (value, mask)
expander.set_output(Port::P0, 0x05, 0x0F).await?;

// Read all input ports
let inputs = expander.read_inputs().await?;
```

## Register map

| Base | Auto-increment | Register |
|------|----------------|----------|
| `0x00` | `0x80` | Input Port 0-2 |
| `0x04` | `0x84` | Output Port 0-2 |
| `0x08` | `0x88` | Polarity Inversion Port 0-2 |
| `0x0C` | `0x8C` | Configuration Port 0-2 |

Shadow registers track output, config, and polarity state to skip redundant I2C writes.

## License

MIT
