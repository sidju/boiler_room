# Boiler room
## A boilerplate++ for a minimal rust web backend

Just fork/clone/download this repo and build your backend on it.

## Why?
Both routing and error handling in rust's web frameworks feel needlessly
abstracted and complex. In an attempt to make a web-backend that feels natural
to me I wrote this boilerplate where `Result<Response,Error>` is cleanly
converted into a correct `Response` and where routing is a tree structure.

## Variants:
- `main` branch, the original "release" for hyper 0.14 with mongodb
- `oidc` branch, a newer variant for hyper 1.0, OIDC auth and postgres via sqlx

## Structure:
- The `main.rs` file. Do I need to tell you? Defines running the application.
- The `state.rs` file defines the state struct shared between async workers
  and its init.
- The `traits.rs` file defines the trait that allows converting from 
  `Result<Response, Error>` to `Response`.
- The `error.rs` file defines Error, and implements the trait from
  `traits.rs` on it.
- The `routes` directory handles everything routing.
  - `mod.rs` starts the routing.
  - The `utils` directory contains utility functions for:
    - `routing.rs` parsing and validating routing
    - `request.rs` parsing and validating request data
    - `response.rs` constructing and modifying responses
  - Other directories under `routes` are new web-routes. The idea is that
    they have a route function that takes `state, req, path_vec` which then
    either returns a response or hands down into another route function.
    All in all resulting in a match tree structure matching the modules.

## How to use:
### Change license:
- Replace `LICENSE.md`
- Change `license` field in `Cargo.toml`

### Adding crates:
- Add to `Cargo.toml`
- Add relevant error types to `error.rs`
- Add relevant variable and init into `state.rs`

### Adding routes:
The routing is intended to be done one "folder" at a time, so adding a route is
easiest when it is one step from an existing one. In that case:
- Create a route function (potentially skip needless arguments)
- Add a match case to the nearest route function for the path you wish to add
- Make that match case run that route function.

If the new path is more than one step from the nearest one you need to make a
choice. Either you create multiple matches to get to the path you want, or you
throw out my routing design in favour of the classic full-path matching.
Especially consider if your path structure is more continuous than not. If more
not, my  tree routing is not for you.

## License
Licensed under either of these:
   - The Unlicense ([LICENSE-UNLICENSE](LICENSE-UNLICENSE.md) or
     https://unlicense.org/)
   - CC0 license ([LICENSE-CC0](LICENSE-CC0.md) or 
     https://creativecommons.org/publicdomain/zero/1.0/)
