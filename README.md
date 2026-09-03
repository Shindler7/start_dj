# Dj — a console utility for running Django applications

At some point, you might ask yourself: why not use uv and tuna to run Django
applications in Python? No problem there. But soon enough, you find yourself juggling a
growing number of command-line parameters that all need to be typed in manually.

You could wrap it all in a .sh script, but that means dealing with unfamiliar,
error-prone syntax, poor readability, and little room for validation. This application
takes care of the heavy lifting, handling all the command-line boilerplate so you don't
have to.

## Built with Rust

This little tool is written in Rust — because why not bring some speed and reliability
to the party? It's a small personal project, but it's sitting right there on GitHub,
open for anyone to use, tweak, or just poke around. If it saves you a few keystrokes
too, feel free to grab it and make it your own.

## Quick Start

Make sure you have `Rust` 1.93 or later installed on your system.

### 1. Clone the repository

```shell
git clone git@github.com:Shindler7/dj.git
cd dj/
```

### 2. Build from source

```shell
cargo build --release
```

The binary will be available at `target/release/dj`.

### 3. Install globally (optional)

If you want to use `dj` from anywhere on your system:

```shell
cargo install --path .
```

This will install the dj binary to your Cargo bin directory (usually `~/.cargo/bin`).
Make sure it's in your `PATH`.

### 4. Set up your project

Create a `start.toml` file in your Django project root. Check out the Configuration
section for a complete example.

> ⚠️ Important: Never hardcode sensitive values (API keys, passwords, etc.) directly in
> start.toml. Instead, use a .env file for your secrets and reference them
> with ${VAR} placeholders — for example, api_key = "${TUNA_API_KEY}". This keeps your
> credentials out of version control.

### 5. Run your Django app

```shell
dj runserver
```
