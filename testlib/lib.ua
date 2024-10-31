# Experimental!

# !doc
# # Hello word
# This is a test file for the extractor.
# It contains a bunch of different comments and annotations.
# It's used to test the extractor and the parser.

~ "git: github.com/ekgame/uiua-essentials" ~ Trim

# This is a function comment.
# It's multiple lines long.
F = +1

# This is a module comment.
# It's multiple lines long.
~MyData {
  Foo: °0type
  Bar: °1type
}

# This is a module comment.
# It's multiple lines long.
┌─╴MyData₂
  ~{
    Foo: °0type
    Bar: °1type
    Baz: °2type
  }
└─╴

# This is a module comment.
# It's multiple lines long.
┌─╴MyData₃
  |Foo {Foo Bar}
  |Bar [r g b]
  |Baz 
└─╴
