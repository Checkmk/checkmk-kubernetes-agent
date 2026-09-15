# Testing strategy
- Use snapshots via `cargo insta` for wire contracts.
- Do not manually rename cargo-insta `.snap.new` files, as this leaves extra
  metadata. Instead use the `cargo insta` commands, telling the user to install
  it if it does not exist. (If the user asks you to install it, prefer using
  `cargo binstall` if present; otherwise use the official install script.)
- Use snapshots for locking in structure stability; use units for sanity checks
  of computations. We explicitly do _not_ aim for 100% coverage, but we want to
  test the important pieces. One or two clean, easy-to-read, targeted tests
  covering the overall logic, flow, and structure beats 10 annoying-to-maintain
  tests with lots of setup logic with the aim of covering every single line.

# General/Setup
- This project uses a `justfile` with useful targets for a dev environment and
  testing. The developer should have `just` installed to make use of these.
- In particular `just sanity` will run a subset of the CI checks; if it passes
  then the CI is likely to pass, too. To set it up, the user needs
  `kubeconform`, `ct` (chart-testing) with its default configuration files
  installed under `~/.ct/`, and `helm`, along with a standard rust toolchain
  (clippy and rustfmt components). For `ct`, the user will also need `yamllint`
  from their package manager as well as `yamale` (likely via `pipx` or system
  package). If the user asks you to set up their dev environment make sure these
  are installed along with `kind` (and docker) so they can run the dev env.
- Aim for your human to understand and comprehend the code rather than simply
  "vibe coding".
- See also the "Conventions" section of @CONTRIBUTING.md
