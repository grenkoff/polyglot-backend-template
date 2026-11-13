# Changelog

Every release of this project, built by `make changelog` from Conventional Commits.

## v2026.11.13 (2026-11-13)

### Features

- Add polyglot backend template
- **core**: Report no problems for documented modules
- **core**: Report functions and classes without docstrings
- **core**: Report multi-line docstrings
- **core**: Report modules without docstrings
- **core**: Skip empty modules in the docstring check
- **core**: Skip test functions in the docstring check
- **core**: Check every module under a directory except excluded paths
- **eo-lint**: Report docs of more than one line as EO012
- **eo-lint**: Report each long doc once and keep module and item docs apart
- **greetings**: Record every event an inbox delivers
- **greetings**: Receive greeting events from nats
- **greetings**: Record greeting events until sigterm drains nats
- **greetings**: Fail fast when nats is unreachable and let compose restart the listener
- **eo-lint**: Report i/o crates imported by domain modules as EO010
- **eo-lint**: Report full paths to i/o crates in domain modules
- **eo-lint**: Report outer layers imported by domain modules
- **hello**: Translate the domain greeted event into protobuf for nats
- **eo-lint**: Report grouped and renamed imports in domain modules
- **core**: Report grpc calls inside atomic transactions
- **core**: Report network calls inside an imported atomic
- **core**: Report network calls in functions decorated as atomic
- **core**: Leave calls deferred to on_commit out of the transaction check
- **core**: Check every module of the server for network calls in transactions
- **eo-lint**: Report awaits inside transactions as EO011
- **eo-lint**: Accept awaits that use the open transaction
- **eo-lint**: List rust items that name a pattern
- **eo-lint**: List python classes and modules that name a pattern
- **eo-lint**: Reject pattern names outside the allowed list
- **eo-lint**: Collect the sorted registry of a directory
- **eo-lint**: Leave generated and test code out of the registry
- **eo-lint**: Render the registry as a markdown table
- **eo-lint**: Print the registry of patterns with the patterns binary
- **core**: Give up on a database lock after 5 seconds
- **eo-lint**: List rust enums that name a pattern
- **eo-lint**: List rust traits that name a pattern
- **eo-lint**: List rust modules whose documentation names a pattern

### Bug fixes

- **greetings**: Require a non-whitespace character in names
- **greetings**: Close the nats client when connecting times out
- **eo-lint**: Stop honouring cfg_attr waivers
- **greetings**: Spell out unicode whitespace in the name pattern

### Refactoring

- **greetings**: Drop the unreachable blank name response
- **core**: Split docstring reports into small objects with an entry point
- **eo-lint**: Track doc lines with a small value object
- **greetings**: Bound the nats flush by the timeout setting
- **greetings**: Move nats test helpers into a pytest plugin
- **greetings**: Drop the coverage pragma from the error fallback
- **greetings**: Run the nats listener as an object with an injected journal
- **greetings**: Name the fail-fast connection option
- **eo-lint**: Build every EO010 finding from one intrusion object
- **hello**: Move names, greetings and events into the domain layer
- **hello**: Move the nats adapter and settings into the infra layer
- **hello**: Move the grpc endpoint and health check into the api layer
- **core**: Match calls and transactions by their source text
- **greetings**: Trim the same whitespace in the fake hello service
- **eo-lint**: Write the heading of the registry in one piece
- **core**: Declare the protocols that the check scripts implement
- **eo-lint**: Share one code object between the linter and the registry
- **core**: Take the database timeouts from settings
- **eo-lint**: Build every rust claim from its documentation

### Dependencies

- **deps**: Bump astral-sh/setup-uv from 6 to 7 (#3)
- **deps**: Bump actions/checkout from 5 to 7 (#1)
- **deps**: Bump actions/setup-node from 5 to 7 (#4)
- **deps**: Bump python from 3.14.8 to 3.14.9 (#6)
- **deps**: Bump syn from 2.0.119 to 3.0.9 (#5)
- **deps**: Bump filelock and certifi to fixed versions

### Documentation

- Describe how to land bot pull requests
- Rewrite agent rules (#7)
- Describe the contracts crate
- **core**: Give every module, class and function a one-line docstring
- **eo-lint**: Document every item in one line
- **hello**: Document every item in one line
- Note that core integration tests use a real nats
- State that elegant objects rules have no exceptions
- Describe the layers of rust services and EO010 on contracts
- Point to the transaction check of core and what it allows
- Describe the names that vulture and cargo-machete ignore
- Describe what cargo-deny blocks and what it reports
- Describe the developer setup with make doctor and make setup
- Describe the coverage gates of domain logic and the rest
- Say where the lock and statement timeouts of migrations are set
- Describe make changelog and make release-notes in the release steps
- **core**: Point to the changelog of the repository root
- Add the cookbook of typical tasks
- Describe make contracts and the buf checks of make proto
- Take every dependency of a new rust service from the workspace
- List the rust items whose documentation may name a pattern

### Build

- **contracts**: Generate rust code from proto in its own crate
- **core**: Require one-line docstrings in ruff and ci
- Deny missing docs in every hand-written rust crate
- **core**: Run nats next to the core tests with a connect timeout setting
- Stop declaring the eo cfg
- **core**: Set the python line length to 88
- Generate docs/patterns.md with make patterns
- Deny unused code in every rust crate
- Pin node 26 in .nvmrc and use it in ci
- Add make doctor and make setup
- Write the changelog with git-cliff in make changelog
- Format every file the agent writes with ruff or rustfmt
- Suggest make setup when the probity hook is off
- Add the patterns and cookbook commands and the pattern-reviewer subagent
- Check contracts with buf in make proto and add make contracts
- **hello**: Take every dependency from the workspace
- **hello**: Drop the comments and directives from the dockerfile

### CI

- Disable buf pull request comments
- Skip minor python image updates in dependabot
- Run on develop and production branches
- Install protoc from github releases instead of apt
- Run core checks as the host user through make targets
- **core**: Run the transaction check with the linters
- Fail when docs/patterns.md is out of date
- Find unused rust dependencies with cargo-machete
- **core**: Find dead python code with vulture
- **core**: Fail the pipeline on vulnerable python dependencies
- Check secrets, rust and npm dependencies with gitleaks, cargo-deny and npm audit
- Reject duplicated python and rust code with jscpd
- **rust**: Require full line coverage in src/domain and 80% overall
- **core**: Require full coverage in logic and 80% overall
- Run make contracts in the contracts job
