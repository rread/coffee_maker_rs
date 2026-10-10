# coffee_maker_rs

A Rust port of
[coffee_maker_ada](https://github.com/c3mnr0nz0yfgvxg4gi0sxhnwl9/coffee_maker_ada),
a tiny 2-word parser game. See that project's README for the background
and `walkthrough.txt` for a solution.

This port models the world with separate `Room`, `Thing` and `Verb` types.
For a port that keeps the original's table-based design, and a version you
can play in a browser, see
[coffee_maker_rust](https://github.com/rread/coffee_maker_rust).

```
cargo run --release
```

## Fidelity

The game logic is meant to be identical to the Ada original: the same
messages (typos included), the same word wrapping, the same pacing (0.1s
per wrapped line, 0.5s before each response), and the same quirks, e.g.
`go nowhere` printing without a trailing newline, or `You are <<noname>>.`
when returning to the library.

Intentional differences, both in how input is read rather than in the game:

- **Whole lines.** The Ada version read into a 100-character buffer, so a
  longer line was split into several commands. This version reads the
  whole line.
- **Clean EOF.** At end of input the Ada version died with an unhandled
  `End_Error`. This version ends the game as `@quit` does.

## Tests

`cargo test` runs unit tests and golden transcripts
(`tests/transcripts/*.in` → `*.out`). The expected outputs were derived
from the Ada source rather than from this program, because no Ada
compiler was available to produce them directly.

## License

GPL-3.0, the same as the Ada original this is ported from. See
[LICENSE](LICENSE).
