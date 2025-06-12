# The NoExport annotation causes a field to not be included in the
# exported JSON:
credentials: {
  username: "root"
  password: @NoExport
  password: "hunter2"
}

# This will be more useful in the next example...