## Yupdates Rust SDK

The Yupdates Rust SDK lets you easily use the Yupdates API from your own software and scripts.

This is [hosted on GitHub](https://github.com/yupdates/yupdates-sdk-rs). There is also a [Yupdates Python SDK](https://github.com/yupdates/yupdates-sdk-py).

### Overview

The `api` module provides a low-level functions that wrap calls to the HTTP+JSON API, serializing and deserializing the requests and responses.

The `clients` module provides an `async` client that is more convenient, and `clients::sync` provides a synchronous version of the client that hides any need to set up an async runtime.

### Getting started

First, obtain an API token from the application. Navigate to "Settings" and then "API".

The examples will start with read-only operations, so you can use the general, read-only token to get started.

Create a new Rust project:

```sh
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
use yupdates::api::YupdatesV0;
use yupdates::clients::sync::new_sync_client;
use yupdates::errors::Error;

fn main() -> Result<(), Error> {
    let yup = new_sync_client()?;
    let response = yup.ping()?;
    println!("Ping worked: {}", response.message);
    Ok(())
}
```

Set the API token environment variable (use a different value, this example will not work):
```sh
set +o history
export YUPDATES_API_TOKEN="789a4e8703:78b15453350458054b84443819060b1a213382cc697a5"
set -o history
```

Test the connection and authentication:
```sh
cargo run
```

Once that is working, you can make other calls. For example, to read the latest 10 items from a feed:

```rust
use yupdates::api::YupdatesV0;
use yupdates::clients::sync::new_sync_client;
use yupdates::errors::Error;

fn main() -> Result<(), Error> {
    let feed_id = "02fb24a4478462a4491067224b66d9a8b2338ddca2737";
    let yup = new_sync_client()?;
    for item in yup.read_items(feed_id)? {
        println!("Title: {}", item.title);
    }
    Ok(())
}
```

There are more examples in the tests and code documentation. You can see the [tests on GitHub](https://github.com/yupdates/yupdates-sdk-rs/tree/main/tests/integration-tests), and see the [code documentation on crates.io](https://docs.rs/yupdates/latest/yupdates/).

### Getting help

You can create a [GitHub issue](https://github.com/yupdates/yupdates-sdk-rs/issues) on this repository for bugs and feature requests.

The fastest way to get help is to create a support ticket from the Yupdates application. Or email `support@yupdates.com`. Especially if you need help that is not specific to this SDK, or if you would like more hands-on setup and troubleshooting advice. 

### License

The SDK is distributed under the MIT license, please see [LICENSE](https://github.com/yupdates/yupdates-sdk-rs/blob/main/LICENSE) for more information.

This covers the source code and examples, but it does not cover related items like the Yupdates logo or API documentation which is not hosted here.
