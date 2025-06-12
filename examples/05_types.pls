# Polsia includes these built-in types:
Types: @NoExport
Types: [
  Any,
  Nothing,
  Int,
  Number,
  Rational,
  Float,
  String,
  Boolean,
]

# There is no distinction between values and types in Polsia; they can
# appear anywhere a value can. They cannot be exported to JSON,
# though, since JSON only deals with values. That is why we set
# @NoExport here.

# Types will successfully unify with values that belong to the type:
pi: Float
pi: 3.141519

# all values (including all types) belong to Any, the "top" type
meaning: Any 
meaning: 42

# no values belong to Nothing, the "bottom" type
error: @NoExport
error: Nothing

# null is a unit type, it does not get a separate type
# (in some language a unit type does get a type, like (): Unit)
npe: null
npe: null