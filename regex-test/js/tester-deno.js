const arg = Deno.args[1]

if (arg === undefined) {
  console.error('error: no argument provided')
  Deno.exit(1)
}

try {
  new RegExp(arg, 'u')
} catch (e) {
  console.error(e.message)
  Deno.exit(1)
}
