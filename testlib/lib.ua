# Experimental!

~ "git: github.com/ekgame/uiua-essentials" ~ Trim

# !doc
# # Headers
#
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
# # Lists
# 
# 1. First ordered list item
# 2. Second ordered list item
# 
# * Unordered list item
# * Another unordered item
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

# The constant containing all letters of the american alphabet in lower case.
Alphabet ← +@a⇡26

# The constant containing all the arabic digits.
Digits ← +@0⇡10

# The constant containing all uppercase and lowercase american alphabet characters
# and the digits.
Alphanumeric ← ⊂⊂ Alphabet ⌵Alphabet Digits

# Noadic function test
# This function does nothing.
DoNothing ← ()

# Generate a random number between 0 and 1.
GenerateRandom ← (⚂)

# Monadic function test
# This function does nothing - takes an argument and returns it.
# output ? input
Identity ← ∘

# Make the first character uppercase.
# result ? input
Capitalize ← ⍣(⍜⊢⌵|∘)

# Test macro
F! ← ^1

# Test macro 2
F‼ ← ^1^2

# Test macro 3
F‼! ← ^1^2^3

# Test macro 4
F‼‼ ← ^1^2^3^4