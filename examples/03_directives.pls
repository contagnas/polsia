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


# noexport can be applied to a nested field
noexport credentials.password
credentials: {
  username: "admin"
  password: "hunter2"
}


trailingCommas: true,

# }
