const arg = process.argv[2]

try {
  new RegExp(arg, 'u')
} catch (e) {
  console.error(e.message)
  process.exit(1)
}
