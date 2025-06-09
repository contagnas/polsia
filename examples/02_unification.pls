### Unification ###
# keys may be duplicated, as long as they don't conflict
simple_string: "string"
simple_string: "string"

# Polsia has types. Types are values which unify with values of that type.
# Built-in types: Any, Nothing, Int, Number, Rational, Float, String, Boolean
funny_number: Int
funny_number: 69

# Object keys are merged together
users: forest: age: 4
users: forest: species: "cat"
