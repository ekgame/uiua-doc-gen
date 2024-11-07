# Experimental!

~ "git: github.com/ekgame/uiua-essentials" ~ Trim

# !doc
# # Headers
# 
# # H1 Header
# ## H2 Header
# ### H3 Header
# #### H4 Header
# ##### H5 Header
# ###### H6 Header
# 
# Alt H1 Header
# ============
# 
# Alt H2 Header
# ------------
# 
# # Text Formatting
# 
# Regular text
# 
# **Bold text**
# 
# *Italic text*
# 
# ***Bold and italic text***
# 
# ~~Strikethrough text~~
# 
# `Inline code`
# 
# # Links and References
# 
# [Basic link](https://www.example.com)
# 
# [Link with title](https://www.example.com "Link title")
# 
# [Reference-style link][arbitrary case-insensitive reference text]
# 
# [Reference link with numbers][1]
# 
# [Relative reference to a repository file](../blob/master/LICENSE)
# 
# URLs and URLs in angle brackets will automatically get turned into links: 
# https://www.example.com or <https://www.example.com>
# 
# [arbitrary case-insensitive reference text]: https://www.example.org
# [1]: https://www.example.com
# 
# # Lists
# 
# 1. First ordered list item
# 2. Second ordered list item
#    1. Nested ordered item
#    2. Another nested ordered item
# 3. Third ordered list item
#    * Unordered sub-list
#    * Another item
# 
# * Unordered list item
# * Another unordered item
#   * Nested unordered item
#     * Deeply nested item
#   * Back one level
# * Back to root level
# 
# - Dashes work too
# + And plus signs
# 
# # Code Blocks
# 
# ```
# Generic code block
# No syntax highlighting
# ```
# 
# ```uiua
# # Make the first character uppercase.
# # result ? input
# Capitalize ← ⍣(⍜⊢⌵|∘)
# ```
# 
# # Blockquotes
# 
# > Single line blockquote
# 
# > Multi-line blockquote
# > with multiple paragraphs
# >
# > Like this
# 
# > Nested blockquotes
# >> Can go deeper
# >>> And deeper still
# 
# # Tables
# 
# | Header 1 | Header 2 | Header 3 |
# |----------|:--------:|---------:|
# | Left     | Center   | Right    |
# | aligned  | aligned  | aligned  |
# | cells    | cells    | cells    |
# 
# | Minimal | Table |
# |---------|-------|
# | Still   | Works |
# 
# # Task Lists
# 
# - [x] Completed task
# - [ ] Incomplete task
# - [x] @mentions, #refs, [links](), **formatting**, and ~~tags~~ supported
# - [ ] Sub-tasks
#   - [x] Completed sub-task
#   - [ ] Incomplete sub-task
# 
# # Line Breaks and Horizontal Rules
# 
# Here's a line break using two spaces at the end  
# And here's the next line

# The constant containing all letters of the american alphabet in lower case.
Alphabet      ← +@a⇡26

# The constant containing all the arabic digits.
Digits        ← +@0⇡10

# The constant containing all uppercase and lowercase american alphabet characters
# and the digits.
Alphanumeric  ← ⊂⊂ Alphabet ⌵Alphabet Digits