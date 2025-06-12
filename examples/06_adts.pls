# Algebraic Data Types (ADTs)
# ADTs are composed of product types (aka records, tuples, objects)
# and sum types (aka unions, enums). 

# In Polsia product types are just objects:
FooAndBar: @NoExport
FooAndBar: {
  foo: Int
  bar: String
}

fooAndBar: FooAndBar
fooAndBar: {
  foo: 3
  bar: "three"
}

# Union types are denoted by "|":
FooOrBar: @NoExport
FooOrBar: { foo: Int } | { bar: String }

a_foo: FooOrBar
a_foo: foo: 3

a_bar: FooOrBar
a_bar: bar: "three"