# mpesa-rust

[![Rust](https://github.com/tralahm/mpesa-rust/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/tralahm/mpesa-rust/actions/workflows/ci.yml)
[![Rust Docs](https://github.com/tralahm/mpesa-rust/actions/workflows/release-core.yml/badge.svg?branch=master)](https://github.com/tralahm/mpesa-rust/actions/workflows/release-core.yml)
[![](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

An unofficial Rust wrapper around the
[Safaricom API](https://developer.safaricom.co.ke/docs?shell#introduction) for
accessing M-Pesa services.

Current release: v2.2.0

> BREAKING CHANGE: The `openssl` crate is now an optional dependency,
> controlled by the `openssl` feature (disabled by default).
> Users targeting platforms without OpenSSL (e.g., musl/Alpine) can now
> build without the system `openssl` libraries.
> Update your Cargo.toml accordingly. See README for details.

With the `no_openssl` feature (enabled by default), the library uses the
[x509-parser](https://crates.io/crates/x509-parser),
[rsa](https://crates.io/crates/rsa),
[base64](https://crates.io/crates/base64), and
[rand](https://crates.io/crates/rand) crates for encrypting client credentials.

Users who want to use [openssl](https://crates.io/crates/openssl) can still
opt in by enabling the `openssl` feature.

**Why this changes?**

- Builds on more platforms (e.g., Alpine Linux, static binaries).
- Enables building on platforms without OpenSSL (e.g., Musl/Alpine).
- Smaller binaries and fewer system dependencies when OpenSSL is not needed.

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
mpesa = { git = "https://github.com/tralahm/mpesa-rust", tag = "v2.2.0" }
```

- By default, the `no_openssl` feature is enabled for compatibility with
  targets like `musl` (e.g., Alpine Linux) where OpenSSL may not available.

```toml
[dependencies]
mpesa = { tag = "v2.2.0", git = "https://github.com/tralahm/mpesa-rust.git" }
```

Optionally, you can disable default-features, which is basically the entire
suite of MPESA APIs to conditionally select individual features.
(See [Services](#services) table for the full list of Cargo features)

Example:

```toml
[dependencies]
mpesa = { tag = "v2.2.0", git = "https://github.com/tralahm/mpesa-rust.git",
    default_features = false,
    features = [
        "b2b",
        "express_request",
        "no_openssl",
    ] }
```

In your lib or binary crate:

```rust,no_run
use mpesa::Mpesa;
```

## Usage

### Creating a `Mpesa` client

You will first need to create an instance of the `Mpesa` instance (the client).
You are required to provide a **CONSUMER_KEY** and **CONSUMER_SECRET**.
[Test Credentials](https://developer.safaricom.co.ke/test_credentials) is how
you can get these credentials for the Safaricom sandbox environment.
It's worth noting that these credentials are only valid in the sandbox
environment. To go live and get production keys read the docs
[Going Live](https://developer.safaricom.co.ke/docs?javascript#going-live).

These are the following ways you can instantiate `Mpesa`:

```rust,no_run
use mpesa::{Environment, Mpesa};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Mpesa::new(
        dotenvy::var("CONSUMER_KEY").unwrap(),
        dotenvy::var("CONSUMER_SECRET").unwrap(),
        Environment::Sandbox,
    );

    assert!(client.is_connected().await);
}
```

Since the `Environment` enum implements `FromStr` and `TryFrom` for `String`
and `&str` types, you can call `Environment::from_str` or
`Environment::try_from` to create an `Environment` type.
This is ideal if the environment values are stored in a `.env` or any
other configuration file:

```rust,no_run
use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

use mpesa::{Environment, Mpesa};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let client = Mpesa::new(
        dotenvy::var("CONSUMER_KEY").unwrap(),
        dotenvy::var("CONSUMER_SECRET").unwrap(),
        Environment::from_str("sandbox")?, /* or
                                            * Environment::try_from("sandbox")?,
                                            */
    );

    assert!(client.is_connected().await);
    Ok(())
}
```

The `Mpesa` struct's `environment` parameter is generic over any type that
implements the `ApiEnvironment` trait. This trait
expects the following methods to be implemented for a given type:

```rust,no_run
pub trait ApiEnvironment {
    fn base_url(&self) -> &str;
    fn get_certificate(&self) -> &str;
}
```

This trait allows you to create your own type to pass to the `environment`
parameter. With this in place, you are able to mock http requests
(for testing purposes) from the MPESA api by returning a mock server uri
from the `base_url` method as well as using your own certificates, required to
sign select requests to the MPESA api, by providing your own
`get_certificate` implementation.

See the example below (and [environment](./src/environment.rs) so see how
the trait is implemented for the `Environment` enum):

```rust,no_run
use mpesa::{ApiEnvironment, Mpesa};

#[derive(Clone)]
pub struct CustomEnvironment;

impl ApiEnvironment for CustomEnvironment {
    fn base_url(&self) -> &str {
        // your base url here
        "https://your_base_url.com"
    }

    fn get_certificate(&self) -> &str {
        // your certificate here
        r#"..."#
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Mpesa::new(
        dotenvy::var("CONSUMER_KEY").unwrap(),
        dotenvy::var("CONSUMER_SECRET").unwrap(),
        CustomEnvironment,
    );
}
```

If you intend to use in production, you will need to call the
`set_initiator_password` method from `Mpesa` after initially
creating the client. Here you provide your initiator password, which overrides
the default password used in sandbox `"Safcom496!"`:

```rust,no_run
use mpesa::{Environment, Mpesa};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Mpesa::new(
        dotenvy::var("CONSUMER_KEY").unwrap(),
        dotenvy::var("CONSUMER_SECRET").unwrap(),
        Environment::Sandbox,
    );

    client.set_initiator_password("new_password");
    assert!(client.is_connected().await)
}
```

### Services

The table below shows all the MPESA APIs from Safaricom and those supported by the crate along with their cargo features and usage examples

| API                                                                                                         | Cargo Feature          | Status        | Example                                                              |
| ----------------------------------------------------------------------------------------------------------- | ---------------------- | ------------- | -------------------------------------------------------------------- |
| [Account Balance](https://developer.safaricom.co.ke/APIs/AccountBalance)                                    | `account_balance`      | Stable ✅     | [account balance example](/docs/client/account_balance.md)           |
| [B2B Express Checkout](https://developer.safaricom.co.ke/APIs/B2BExpressCheckout)                           | N/A                    | Unimplemented | N/A                                                                  |
| [Bill Manager](https://developer.safaricom.co.ke/APIs/BillManager)                                          | `bill_manager`         | Unstable ⚠️   | [bill manager examples](/docs/client/bill_manager/)                  |
| [Business Buy Goods](https://developer.safaricom.co.ke/APIs/BusinessBuyGoods)                               | `b2b`                  | Stable ✅     | [business buy goods example](/docs/client/b2b.md)                    |
| [Business Pay Bill](https://developer.safaricom.co.ke/APIs/BusinessPayBill)                                 | N/A                    | Unimplemented | N/A                                                                  |
| [Business To Customer (B2C)](https://developer.safaricom.co.ke/APIs/BusinessToCustomer)                     | `b2c`                  | Stable ✅️     | [b2c example](/docs/client/b2c.md)                                   |
| [Customer To Business (Register URL)](https://developer.safaricom.co.ke/APIs/CustomerToBusinessRegisterURL) | `c2b_register`         | Stable ✅️     | [c2b register example](/docs/client/c2b_register.md)                 |
| [Customer To Business (Simulate)](#)                                                                        | `c2b_simulate`         | Stable ✅️     | [c2b simulate example](/docs/client/c2b_simulate.md)                 |
| [Dynamic QR](https://developer.safaricom.co.ke/APIs/DynamicQRCode)                                          | `dynamic_qr`           | Stable ✅️     | [dynamic qr example](/docs/client/dynamic_qr.md)                     |
| [M-PESA Express (Query)](https://developer.safaricom.co.ke/APIs/MpesaExpressQuery)                          | `express`              | Stable ✅️ ️    | [express query example](/docs/client/express.md)                     |
| [M-PESA Express (Simulate)/ STK push](https://developer.safaricom.co.ke/APIs/MpesaExpressSimulate)          | `express`              | Stable ✅️     | [express request example](/docs/client/express.md)                   |
| [Transaction Status](https://developer.safaricom.co.ke/APIs/TransactionStatus)                              | `transaction_status`   | Stable ✅️     | [transaction status example](/docs/client/transaction_status.md)     |
| [Transaction Reversal](https://developer.safaricom.co.ke/APIs/Reversal)                                     | `transaction_reversal` | Stable ✅️     | [transaction reversal example](/docs/client/transaction_reversal.md) |
| [Tax Remittance](https://developer.safaricom.co.ke/APIs/TaxRemittance)                                      | N/A                    | Unimplemented | N/A                                                                  |

## Original Author

**Collins Muriuki**

- Github: [@c12i](https://github.com/c12i)
- Not affiliated with Safaricom.

## Maintainer

**Tralah M Brian**

- Github: [@tralahm](https://github.com/tralahm)

## Contributing

Contributions, issues and feature requests are welcome!<br/>
Feel free to check [issues page](https://github.com/tralahm/mpesa-rust/issues).
You can also take a look at the
[contributing guide](https://raw.githubusercontent.com/tralahm/mpesa-rust/master/CONTRIBUTING.md).

<a href="https://github.com/tralahm/mpesa-rust/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=tralahm/mpesa-rust" alt="Contributors" />
</a>

Made with [contrib.rocks](https://contrib.rocks).

---

Copyright © 2026 [Tralah M Brian](https://github.com/tralahm).<br />
This project is
[MIT](https://raw.githubusercontent.com/tralahm/mpesa-rust/master/LICENSE) licensed.
