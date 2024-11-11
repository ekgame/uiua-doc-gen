# uiua doc gen

This is a command line tool to generate documentation from your [Uiua](https://uiua.org/) libraries.

# Limitations

This tool is still in development and has some limitations:
- Only the declarations in `lib.ua` files are considered. If you import bindings from other files, they will not be displayed in this version.
- Generated cods contain source code for the declarations, but it isn't highlighted yet.
- There's only one theme available for the generated documentation.