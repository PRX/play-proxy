[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Creates an HTTP server intended to handle requests with a `url` query parameter. For example, if the server is running at `example.com`, it would handle a request like `example.com/?url=http://other.example.com/some-page.

The server makes a GET request to the `url` provided, and responds with the body content of that request, in order to proxy the contents. Headers/etc are not proxies.

You can build the program with `cargo build`, and then run it locally with `cd target/debug && ./play_proxy`.
