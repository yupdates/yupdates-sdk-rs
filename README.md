## Yupdates Rust SDK

The Yupdates Rust SDK lets you easily use the Yupdates API from your own software and scripts.

Also see the [Yupdates Python SDK](https://github.com/yupdates/yupdates-sdk-py).

### Getting started

First, obtain an API token from the application. Navigate to "Settings" and then "API".

The examples will start with read-only operations, so you can use the general, read-only token to get started.

Create a new Rust project:

```shell
$ cargo new yupdates-example
$ cd yupdates-example
```

Add the dependency to `Cargo.toml`:
```toml
[dependencies]
yupdates = "0"
```

Add this to the `src/main.rs` file:
```rust
use std::process;
use yupdates::api::{PingResponse, YupdatesV0};
use yupdates::clients::sync::new_sync_client;
use yupdates::errors::Result;

fn main() {
    match ping_one() {
        Ok(ping_response) => {
            println!("Worked: {}", ping_response.message);
        }
        Err(e) => {
            eprintln!("Failed. {}", e);
            process::exit(1);
        }
    }
}

fn ping_one() -> Result<PingResponse> {
    let yup = new_sync_client()?;
    yup.ping()
}
```

Set the API token environment variable:
```shell
set +o history
export YUPDATES_API_TOKEN="1a7814fc25:c38bb526..."
set -o history
```

During the preview, you also need to set the URL. This won't be necessary once there is a default endpoint.
```shell
export YUPDATES_API_URL="https://..."
```

Test the connection and authentication:
```shell
cargo run
```

### Getting help

You can create a [GitHub issue](https://github.com/yupdates/yupdates-sdk-rs/issues) on this repository for bugs and feature requests.

The fastest way to get help is to create a support ticket from the Yupdates application. Or email `support@yupdates.com`. Especially if you need help that is not specific to this SDK.

### License

The SDK is distributed under the MIT license, please see [LICENSE](https://github.com/yupdates/yupdates-sdk-rs/blob/main/LICENSE) for more information.

This covers the source code and examples, but it does not cover related items like the Yupdates logo or API documentation which is not hosted here.
