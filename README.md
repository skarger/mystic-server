# Code Organization

We use [Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to separate logical components of the code.
The `db` and `web` workspaces both include code to deal with external boundaries of the application, that is, database access and the web API.

As the application grows we may want one or more other workspaces, for example one dedicated to domain-specific logic.

# Development Environment

## Environment Variables

For local development we use [dotenv](https://crates.io/crates/dotenv).
This loads values from the `.env` file. We do not store actual secrets in this file, only safe-to-expose development environment values.

## Running the web server

To run with automatic rebuilding on code changes, use this command:

`systemfd --no-pid -s http::8080 -- cargo watch -x run`

The standard `cargo run` works too.
