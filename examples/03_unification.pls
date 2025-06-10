# If you're coming from the previous demo, you might have noticed that
hello: "world"
goodbye: "moon"

# could be interpreted as
{
  "hello": "world",
  "goodbye": "moon"
}

# but maybe also as
{ "hello": "world" }
{ "goodbye": "moon" }

# Or maybe now you are noticing that we have defined "hello" and
# "goodbye" three times here! So you might wonder, which one wins? The
# answer is all of them: Polsia "unifies" these values. Keys may be
# defined multiple times, as long as their values agree.

# Unification on objects merges their keys together. This happens
# recursively, so can create (or patch) deeply nested objects like so:
foo: bar: baz: string: "hello"
foo: bar: baz: int: 3