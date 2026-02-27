# rush — Development Roadmap

A prioritized plan for building out the `rush` shell, ordered from easiest to hardest.
Each step teaches new Rust concepts while making the shell more realistic and functional.

---

## Current State

- **Builtins:** `exit`, `echo`, `type`, `pwd`, `cd` (with tilde expansion)
- **External commands:** PATH lookup and execution via `Command::new`
- **Argument splitting:** `split_whitespace` (no quote handling)
- **Cross-platform:** Unix/Windows executable detection and path handling
- **Dependencies:** `anyhow` (present but unused)

---

## Step 1 — Quoted String Parsing & Escape Handling

> **Difficulty:** ⭐ | **Impact:** High

Replace `split_whitespace` with a hand-written tokenizer that handles:

- Single quotes (`'hello world'` → one token)
- Double quotes (`"hello world"` → one token)
- Backslash escapes (`\"`, `\\`, `\ `)

**Why:** `echo "hello world"` currently breaks into `"hello` and `world"`. This is the most immediate usability fix.

**Rust concepts:** `char` iterators, `match` expressions, `Vec<String>` building, unit testing with `#[cfg(test)]`.

---

## Step 2 — Refactor Builtins with Enums, Traits & Modules

> **Difficulty:** ⭐⭐ | **Impact:** Medium (code quality)

- Replace the big `match command { ... }` block with a `Builtin` enum and a `trait Executable`.
- Move each builtin into its own file under `src/builtins/` (`cd.rs`, `echo.rs`, etc.).
- Start using `anyhow::Result` with `?` instead of `unwrap()`.

**Why:** The single `main.rs` will become unmanageable as features grow. This restructures the project early.

**Rust concepts:** enums, traits, `impl` blocks, module system (`mod` / `use`), `Result` / `?` error handling.

---

## Step 3 — I/O Redirection (`>`, `>>`, `<`, `2>`)

> **Difficulty:** ⭐⭐ | **Impact:** High

- Parse redirect operators out of the token list.
- Open files with `std::fs::File` / `OpenOptions`.
- Wire them into `Command::new().stdin()` / `.stdout()` / `.stderr()`.
- Add a `Redirect` struct to hold the fd, mode, and path.

**Rust concepts:** `File` ownership (the `Stdio::from` call *takes ownership*), `OpenOptions` builder, enums for redirect mode (Overwrite / Append).

---

## Step 4 — Pipelines (`cmd1 | cmd2 | cmd3`)

> **Difficulty:** ⭐⭐⭐ | **Impact:** High

- Split input on unquoted `|`.
- Spawn each command with `Stdio::piped()`.
- Chain each child's stdout to the next child's stdin.
- Wait on all children; propagate the last exit code.

**Rust concepts:** ownership transfer of `ChildStdout` → `Stdio`, `Vec<Child>`, `.windows(2)` iteration, process lifecycle management.

---

## Step 5 — Environment Variable Expansion & `export` / `unset`

> **Difficulty:** ⭐⭐⭐ | **Impact:** Medium

- Expand `$VAR` and `${VAR}` in tokens before execution.
- Add `export KEY=VALUE` builtin (sets in local `HashMap` and via `env::set_var`).
- Add `unset` builtin.
- Introduce a `struct Shell` to hold all shell state (env, builtins, etc.).

**Rust concepts:** `HashMap`, `Entry` API, `&str` vs `String` borrowing, structuring program state in a struct.

---

## Step 6 — Signal Handling & Job Control (`Ctrl-C`, `&`, `jobs`)

> **Difficulty:** ⭐⭐⭐⭐ | **Impact:** Medium

- Use the `signal-hook` crate so Ctrl-C kills the foreground child, not the shell.
- Add `&` suffix for background execution.
- Add `jobs`, `fg`, `bg` builtins, tracking children in a `Vec`.

**Rust concepts:** `Arc<AtomicBool>`, cross-thread signaling, `unsafe` awareness (signal handlers), crate integration, process groups.

---

## Step 7 — Line Editing & History with `rustyline`

> **Difficulty:** ⭐⭐⭐⭐ | **Impact:** High (UX)

- Replace raw `stdin().read_line` with the `rustyline` crate.
- Get arrow-key navigation, history search (Ctrl-R), persistent `~/.rush_history`.
- Implement the `Helper` trait for tab-completion of builtins, file paths, and PATH commands.

**Rust concepts:** trait implementation (`Completer`, `Hinter`, `Highlighter`, `Validator`), generics, lifetimes in trait impls.

---

## Step 8 — Script Execution & Control Flow (`if`, `for`, `while`)

> **Difficulty:** ⭐⭐⭐⭐⭐ | **Impact:** High

- Accept `rush script.sh` via `std::env::args()`, read the file, execute line-by-line.
- Build a simple AST:
  ```rust
  enum Statement {
      Simple(Command),
      If { cond: Box<Statement>, then: Vec<Statement>, else_: Option<Vec<Statement>> },
      For { var: String, list: Vec<String>, body: Vec<Statement> },
  }
  ```
- Write a recursive `eval` function. Use exit codes as booleans.

**Rust concepts:** recursive enums with `Box<T>`, pattern matching on nested structures, `Iterator` for file lines, tree-walking interpretation.

---

## Cross-Cutting Concerns

| Concern | Notes |
|---|---|
| **Shell state struct** | Steps 2–8 all benefit from a `struct Shell { builtins, env, jobs, history, ... }`. Introduce it in Step 2. |
| **Testing** | The tokenizer (Step 1) and variable expansion (Step 5) are pure functions — ideal for `#[test]` modules. Use the `assert_cmd` crate for integration tests. |
| **`anyhow` usage** | Already a dependency but unused. Every step is a chance to replace `unwrap()` with `?`. |
| **Error messages** | Follow the Unix convention: `command: context: error message`. |

---

*Last updated: 2025-02-26*

