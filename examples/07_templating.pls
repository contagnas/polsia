# Polsia doesn't support templating, but its unification operates a
# bit like inheritance and can be used like templating.

noexport Bear
Bear: {
  species: "bear"
  says: "roar"
}

noexport Dog
Dog: {
  species: "dog"
  says: "bark"
}

noexport Cat
Cat: {
  species: "cat"
  says: "meow"
}

animals: {
  meadow: Bear
  meadow: color: "black"

  forest: Cat
  forest: coward: true

  pluto: Dog
  pluto: planet: false
}

# Unification can do some deduction
noexport Pet
Pet: Dog | Cat
pet: Pet
pet: says: "meow"
# pet.species must be cat, Polsia deduces it.