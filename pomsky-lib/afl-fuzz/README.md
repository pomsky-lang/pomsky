# AFL fuzzer

This fuzzer checks that the Pomsky compiler does not crash for any input, and produces valid regular expressions.

The latter requirement is tested by compiling the regex with the respective regex engine. This requires the following programs to be installed:

- deno (for JavaScript)
- javac
- python
- mcs (for .NET)

## Usage

It is recommended to use [just](https://github.com/casey/just). When fuzzing Pomsky for the first time, run

```sh
just fuzz_init
just fuzz in
```

When you want to resume a previous fuzzing session, you can just

```sh
just fuzz
```

## Analyze crashes

When you found a crash, you might find it in `errors.txt`. If it's not in `errors.txt`, that likely means that there was an unexpected panic. To minimize it, run `just tmin <path>`, where `<path>` is the path to a file in the `out/default/crashes` folder. This command minimizes the input for the crash and creates a logfile at `log.txt` that should make it possible to identify the bug.

## Report the bug

Please report the bug [here](https://github.com/pomsky-lang/pomsky/issues). If you think it could be a security vulnerability, please disclose it directly per email: ludwig.stecher@gmx.de.
