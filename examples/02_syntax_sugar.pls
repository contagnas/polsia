# Polsia has some syntax sugar to make data less verbose.

# You may have already noticed: there are comments, which are notated
# with '#' 

# A file containing an object does not need braces:
"hello": "world",

# Object keys do not need quotes:
goodbye: "moon",

# Commas are not required in objects:
users: {
  forest: "cat"
  meadow: "bear"
}

# Nested objects with a single key can omit braces
foo: bar: "baz"

# Trailing commas are permitted
numbers: [
  0,
  1,
  2,
],