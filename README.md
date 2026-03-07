# Scribble

**Scribble** is a minimal terminal text editor written in Rust.
It is lightweight, keyboard-driven, and designed to run entirely inside the terminal.

Scribble focuses on simplicity and speed while providing useful editing features like highlighting, clipboard actions, and undo/redo.

---

## Features

* Minimal terminal UI
* Insert mode editing
* Cursor navigation with `h j k l`
* Copy / Cut / Paste
* Undo / Redo
* Text highlighting
* File saving
* Lightweight and fast
* Works entirely in the terminal

---

## Installation

Install directly from Cargo:

```bash
cargo install scribble
```

---

## Usage

Open a file:

```bash
scribble filename.txt
```

If the file does not exist, Scribble will create it when you save.

---

## Editor Controls

| Key             | Action              |
| --------------- | ------------------- |
| `SPACE`         | Enter insert mode   |
| `ESC`           | Exit insert mode    |
| `h` `j` `k` `l` | Move cursor         |
| `hh`            | Toggle highlight    |
| `c`             | Copy                |
| `v`             | Paste               |
| `x`             | Cut                 |
| `u`             | Undo                |
| `r`             | Redo                |
| `s`             | Save                |
| `q`             | Quit                |
| `t`             | Move to first line  |
| `b`             | Move to last line   |


---

## Philosophy

Scribble follows a few simple principles:

* **Keyboard first**
* **Minimal interface**
* **Fast startup**
* **Terminal native**

It is designed for users who prefer quick edits without leaving the terminal.

---

## Building from Source

Clone the repository:

```bash
git clone https://github.com/SinofPride-999/Scribble
cd Scribble
```

Build the project:

```bash
cargo build --release
```

Run the editor:

```bash
cargo run -- filename.txt
```

Or:

```bash
./target/release/scribble filename.txt
```

---

## Project Structure

```
src/
 ├── app.rs
 ├── buffer.rs
 ├── clipboard.rs
 ├── cursor.rs
 ├── editor.rs
 ├── file_io.rs
 ├── highlight.rs
 ├── history.rs
 ├── input.rs
 ├── mode.rs
 ├── renderer.rs
 └── status.rs
```

---

## Contributing

Contributions are welcome!
Please read **CONTRIBUTION.md** for guidelines.

---

## License

This project is licensed under the **MIT License**.