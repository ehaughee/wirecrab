---
description: Instructions for GitHub Copilot on how to assist with the Wirecrab project.
applyTo: **
---

When generating code for the Wirecrab project, please adhere to the following guidelines:

1. This project is written in Rust, so ensure that any code suggestions adhere to Rust syntax and conventions.
2. The project structure includes modules for parsing, loading, GUI, TUI, and flow management. Make sure to respect this modular architecture when adding or modifying code. But do suggest removing, changing, or adding modules as needed.
3. The project uses a combination of synchronous and asynchronous programming patterns. Be mindful of which context you are working in when suggesting code.
4. The project includes tests for various components. Ensure that any new code is accompanied by appropriate tests.
5. The project appears to handle network packet data, so be cautious about performance and memory management when dealing with large datasets.
6. Follow existing coding styles and patterns used throughout the codebase to maintain consistency.
7. Ensure that any changes made do not break existing functionality unless explicitly intended.
8. When asked to commit changes, refer to previous commit messages for style and format.
9. Reference https://longbridge.github.io/gpui-component/docs/components/ to look for componenets that can be reused in the project before generating new ones.
10. When working on UI components, consider both the TUI and GUI aspects of the application to ensure a consistent user experience across interfaces. Ask if the user wants changes to both or just one. Often we will defer changes to the TUI until later or indefinitely if they are not a good fit.
11. When dealing with network protocols or data structures, ensure that any parsing or serialization logic adheres to relevant standards and best practices.
12. After every change, check the results with `cargo check --features ui,tui` to ensure that both UI and TUI features are correctly handled.
