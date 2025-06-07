# Polsia (Edit me!)
# https://github.com/contagnas/polsia
#
# Polsia is a data/configuration language, similar to CUE,
# and a superset of JSON

### Syntax sugar features ###
# Comments start with #
# this is a comment

# braces may be skipped in a top-level object
# {

"hello": "world",

# quotes are optional for object keys
goodbye: "moon",

# commas are optional in objects
commas: "optional"

# braces are optional for objects with a single key
foo: bar: baz: "nested"

### Unification ###
# keys may be duplicated, as long as they don't conflict
simple_string: "string"
simple_string: "string"

# Polsia has types. Types are values which unify with values of that type.
# Built-in types: Any, Nothing, Int, Number, Rational, Float, String
funny_number: Int
funny_number: 69

# Object keys are merged together
users: forest: age: 4
users: forest: species: "cat"

### Directives
# Some functionality is controled by directives.

# noexport prevents a field from being exported.
noexport creature
creature: {
  # Without noexport, these underspecified fields would cause
  # the JSON export to fail.
  age: Int
  species: String
}

users: meadow: creature
users: meadow: age: 4
users: meadow: species: "bear"

users: dmed: creature
users: dmed: {
  age: Int
  age: 1e100
  species: "Doctor"
}

# noexport can also be used to create templates
noexport bear
bear: says: "roar"
users: meadow: bear


# Example: add restraitns to the template
bear: species: "bear"
# users: dmed: bear # doesn't match, causes an error


trailingCommas: true,

# }
