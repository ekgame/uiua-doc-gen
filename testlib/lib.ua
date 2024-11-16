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

Matrix ← °△3_3

Num ← ℂ0 0

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

# Extract words from an arbitrary string.
# Mostly used to transform strings to different formats.
# boxedArray ? string
ExtractWords ← (
  # Replace non-alphanumeric characters with spaces
  ⍜▽(⋅@\s)⊸(¬∈Alphanumeric)
  # Insert a space in word boundries
  ⍜⊜∘≡(⊂⊂:" "°⊟)⦷[1 0]≠⌵..
  # Split words by spaces
  ⊜(□¯⌵)≠" ".
)

# Test macro
F! ← ^1

# Test macro 2
F‼ ← ^1^2

# Test macro 3
F‼! ← ^1^2^3

# Test macro 5
Enum! ←^ ⍚($"_ ← $_ _\n".) :⇡⧻.⍚(⊂∩°□)°⊂

┌─╴TestModule
  # Variant without fields.
  |Variant 
  # Variant unboxed with fields.
  |AnotherVariant [This Ones Unboxed]
  
  # Variant boxed with fields.
  |ThirdVariant {Boxed This Time}
└─╴

# Test module description.
┌─╴TestModule
  # Test comment
  ~[Unnamed Data Definition]
  
  # Test comment 2
  ~TestDataNotBoxed [Not Boxed]
  
  # Test comment 3
  ~TestDataBoxed {Boxed Data}
  
  # Test comment 4
  ~TestDataWithValidators {
    Boxed: °0type
    Data: °1type
    With: °2type
    Validators: °3type
    StringType: °1⧻△
    MoreThanThreeType: °1>3
  }
  
  ModuleFunction  ← ∘
  ModuleFunction₂ ← ∘
  
  ┌─╴SubModule
    ModuleFunction  ← ∘
    ModuleFunction₂ ← ∘
  └─╴
└─╴

# Test module with only private items, should not be generated.
┌─╴TestModule₂
  ModuleFunction₃ ↚ ∘
  ModuleFunction₄ ↚ ∘
└─╴

# Test comment
~[Unnamed Data Definition]

# Test comment 2
~TestDataNotBoxed [Not Boxed]

# Test comment 3
~TestDataBoxed {Boxed Data}

# Test comment 4
~TestDataWithValidators {
  Boxed: °0type
  Data: °1type
  With: °2type
  Validators: °3type
  StringType: °1⧻△
  MoreThanThreeType: °1>3
}
