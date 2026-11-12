# Elegant Objects in this repository

We follow [Elegant Objects](https://www.elegantobjects.org/) by Yegor Bugayenko:

- **Rust: 100%.** Every rule below is mandatory and checked in CI.
- **Python: where it does not fight Pythonic style or Django.** The framework
  wins when it requires something (models, managers, `Meta`, settings,
  management commands). Our own code follows the rules.

## The rules

| # | Rule | Rust | Python |
|---|------|------|--------|
| 1 | **An object is something, not someone who does something.** No `-er`/`-or` class names: `Manager`, `Handler`, `Helper`, `Util`, `Processor`, `Validator`, `Parser`. Name the object after what it *is*: `Trimmed`, `Polite`, `Greetings`, `Inbox`. | `eo-lint` EO001 | review |
| 2 | **No getters and setters.** Methods are named after what they return (`text()`, `age()`) or what they do (`publish()`). Never `get_*`/`set_*`. | EO002 | review |
| 3 | **Objects are immutable.** State is set in the constructor and never changes. | EO003: no `&mut self` outside trait impls | `@attrs.define(frozen=True)`, `msgspec.Struct(frozen=True)` |
| 4 | **Every public method implements an interface.** Callers depend on traits/protocols, never on concrete types. | EO004 | `typing.Protocol` for every collaborator |
| 5 | **No procedures and no utility classes.** Behaviour belongs to objects. | EO005: free functions only `main` | module-level functions only where Django requires them |
| 6 | **No public data.** Objects expose behaviour, not fields. | EO006 | private attributes (`_name`); DTOs at the API boundary are the exception |
| 7 | **No global state and no singletons.** Pass dependencies through constructors. | EO007: no `static` | DI container (`punq`), no module-level mutable objects |
| 8 | **Constructors only assign.** No code, no I/O, no validation in constructors; static methods are forbidden. | EO008 | `attrs` classes without custom `__init__`/`__attrs_post_init__` |
| 9 | **No reflection and no type checks.** Use polymorphism instead of `downcast`/`isinstance`. | EO009 | avoid `isinstance` in business logic |
| 10 | **No `NULL`.** Absence is explicit. | `Option`/`Result`, `unwrap`/`expect`/`panic` are denied by clippy | raise exceptions or return empty collections; `None` only for optional input |
| 11 | **Composition over inheritance; decorators over flags.** Add behaviour by wrapping an object into another object that implements the same interface. | no implementation inheritance in Rust | `@final` on every class; inherit only from framework bases and protocols |
| 12 | **Small objects.** Few fields (up to 4), short methods. | clippy `too_many_lines`, `too_many_arguments` | wemake-python-styleguide complexity rules |
| 13 | **Fail fast.** Errors surface immediately and are typed. | `thiserror` errors, `?` | specific exception classes |

## Decorators instead of flags

```rust
// Bad: a flag inside one object.
greeting.text(polite: true)

// Good: features are objects that wrap other objects.
Polite::new(Plain::new(Trimmed::new(Raw::new(name)))).text()
```

See `services/hello/src/domain/greeting.rs` and
`services/hello/src/domain/name.rs`.

## No exceptions

There is no way to waive a rule: no attribute, no directive, no
configuration. When a framework or a crate seems to leave no choice, stop
and ask the human, as "Elegant Objects" in `AGENTS.md` describes.

## What clippy already checks

`Cargo.toml` (`[workspace.lints]`) denies `clippy::pedantic`, `unwrap_used`,
`expect_used`, `panic`, `todo`, `print_stdout` and limits function length
(`clippy.toml`). `unsafe` is forbidden.
