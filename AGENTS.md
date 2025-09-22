# EVMap OSM Loader

This program downloads all charging stations from OpenStreetMap via Overpass
API, preprocesses and aggregates them and exports them as compressed JSON file.

Please see the `README.md` file for more information.

## Build & Commands

- Run typechecking: `cargo check`
- Build binary: `cargo build`
- Run tests: `cargo test`
- Format code: `cargo fmt`
- Run linting: `cargo clippy`

## Code Style

- Follow Rust coding conventions
- Apply Rust code style using `cargo fmt`
- Use crate import groups separated by an empty line: std, 3rd party, 1st party
- Use merged imports: One `use` per crate
- Avoid deeply nested logic by using early returns for error cases
- Write clean, high-quality code with concise comments and clear variable names

## Security

- Never commit secrets or API keys to repository

## Decisions

Whenever there is a situation where you need to choose between two or more
approaches, don't just pick one. Instead, ask.

This includes:

- Choosing between two possible architectural approaches
- Choosing between two libraries to use
...and similar situations.
