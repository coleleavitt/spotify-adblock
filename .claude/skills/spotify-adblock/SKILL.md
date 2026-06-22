```markdown
# spotify-adblock Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill covers the core development patterns and conventions used in the `spotify-adblock` Rust codebase. It is designed to help contributors quickly understand the project's structure, coding style, and common workflows. The repository focuses on blocking ads in Spotify using Rust, without reliance on any specific frameworks.

## Coding Conventions

### File Naming
- Use **camelCase** for file names.
  - Example: `adBlocker.rs`, `spotifyClient.rs`

### Import Style
- Use **relative imports** within the project.
  - Example:
    ```rust
    mod utils;
    use crate::adBlocker::AdBlocker;
    ```

### Export Style
- Use **named exports**.
  - Example:
    ```rust
    pub struct AdBlocker { ... }
    pub fn block_ads() { ... }
    ```

### Commit Messages
- Freeform style, no enforced prefixes.
- Average commit message length: ~35 characters.
  - Example: `Fix ad detection logic`

## Workflows

### Building the Project
**Trigger:** When you want to compile the project to check for errors or produce a binary.
**Command:** `/build`

1. Open a terminal in the project root.
2. Run:
   ```bash
   cargo build
   ```
3. Check for compilation errors and warnings.

### Running the Project
**Trigger:** To execute the adblocker and test its functionality.
**Command:** `/run`

1. Ensure you have built the project.
2. Run:
   ```bash
   cargo run
   ```

### Adding a New Feature
**Trigger:** When implementing a new ad-blocking strategy or feature.
**Command:** `/add-feature`

1. Create a new camelCase file for your feature (e.g., `smartBlocker.rs`).
2. Implement your logic using relative imports as needed.
3. Export structs/functions with `pub`.
4. Add usage in the main module or integrate with existing logic.
5. Commit your changes with a clear, concise message.

### Writing Tests
**Trigger:** When adding or updating functionality that requires verification.
**Command:** `/test`

1. Create a test file following the pattern `*.test.*` (e.g., `adBlocker.test.rs`).
2. Write test functions using Rust's built-in test framework.
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_block_ads() {
           // test logic here
       }
   }
   ```
3. Run tests with:
   ```bash
   cargo test
   ```

## Testing Patterns

- Tests are placed in files matching `*.test.*` (e.g., `adBlocker.test.rs`).
- Use Rust's built-in `#[test]` attribute for test functions.
- Example:
  ```rust
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_ad_blocking() {
          // Arrange
          // Act
          // Assert
      }
  }
  ```
- Testing framework is not explicitly defined; default Rust test runner is assumed.

## Commands
| Command      | Purpose                                      |
|--------------|----------------------------------------------|
| /build       | Compile the project                          |
| /run         | Run the main application                     |
| /add-feature | Scaffold and add a new feature module        |
| /test        | Run all tests in the project                 |
```