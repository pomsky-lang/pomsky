const arg = process.argv[2]

if (arg === undefined) {
  console.error('error: no argument provided')
  process.exit(1)
}

try {
  new RegExp(arg, 'u')
} catch (e) {
  console.error(e.message)
  process.exit(1)
}
