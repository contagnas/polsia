# The NoExport annotation causes a field to not be included in the
# exported JSON:
credentials.password: @NoExport
credentials: {
  username: "root"
  password: "hunter2"
}

# This will be more useful in the next example...