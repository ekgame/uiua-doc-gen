# uiua doc gen

This is a command line tool to generate documentation from your [Uiua](https://uiua.org/) libraries.

# Limitations

This tool is still in development and has some limitations:
- Only the declarations in `lib.ua` files are considered. If you import bindings from other files, they will not be displayed in this version.
- Generated cods contain source code for the declarations, but it isn't highlighted yet.
- There's only one theme available for the generated documentation.
- Can not embed images yet.

# Prerequisites

- You need to have [Rust](https://www.rust-lang.org/) installed in your system.

# Usage
1. Install the package globally:
    ```bash
    cargo install uiua-doc-gen
    ```

2. Open a terminal in the root of your Uiua project and run:
    ```bash
    uiua-doc-gen --name project-name
    ```
   
3. The documentation will be generated in the `doc-site` folder.