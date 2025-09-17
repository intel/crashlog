# Lightweight Crash Log Framework - Command Line Interface

This command-line application is designed to extract and decode Crash Log
records directly from the terminal.

## Building

To build and install the application, follow these steps:

1. **Build the Application in Release Mode:**
  
  ```
  $ rustup default nightly
  $ cargo build --release
  ```

2. **Install the application:**

  ```
  $ cargo install --path .
  ```

3. **Uninstalling the application:**

  ```
  $ cargo uninstall
  ```

## Usage

For detailed usage instructions, please refer to the
[main README](../README.md#Usage).

## Development

To build and test changes, use the following commands:

```
cargo build
cargo run
```

Before submitting pull requests that modify any files in this directory, please
ensure the following:

1. **Cross-Platform Build Verification:**

  Verify that the application builds successfully on both Windows and Linux:

  ```
  $ cargo build --target=x86_64-unknown-linux-gnu
  $ cargo build --target=x86_64-pc-windows-gnu
  ```

2. **Code Formatting and Linting:**

  Format the code according to the style guidelines and run the linter:

  ```
  $ cargo fmt
  $ cargo clippy
  ```
