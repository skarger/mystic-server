`systemfd --no-pid -s http::8080 -- cargo watch -x run`

# Development Environment

## Environment Variables

For local development we use [dotenv](https://crates.io/crates/dotenv).
This loads values from the `.env` file. We do not store actual secrets in this file, only safe-to-expose development environment values.
