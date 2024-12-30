set -l features \
'ascii-mode atomic-groups boundaries dot grapheme intersection lazy-mode lookahead lookbehind named-groups numbered-groups ranges recursion references regexes variables'

set -l flavors \
'pcre	PCRE flavor
python	Python re flavor
java	Java flavor
js		JavaScript (ECMAScript) flavor
dotnet	C# (.NET) flavor
ruby	Ruby (Oniguruma) flavor
rust	Rust regex flavor
re2		RE2 flavor'

set -l warnings \
'0				Disable all warnings
compat=0		Disable compatibility warnings
deprecated=0	Disable deprecation warnings'

set -l engines \
'pcre2	PCRE2 regex engine
rust	Rust crate `regex`'

set -l subcommands \
'test	Run unit test suite
'

# subcommands
complete -c pomsky -n "not __fish_seen_subcommand_from test" -fa "(echo \"$subcommands\")"
# disable file completions (-f) for `pomsky test`
complete -c pomsky -f -n '__fish_seen_subcommand_from test'

# global args
complete -c pomsky      -l allowed-features -d 'Allowed features, comma-separated' -xa "(__fish_append , $features)"
complete -c pomsky -s f -l flavor           -d 'Regex flavor' -xa "(echo \"$flavors\")"
complete -c pomsky -s h -l help             -d 'Show help information'
complete -c pomsky -s p -l path             -d 'File or directory' -kxa "(__fish_complete_suffix .pomsky)"
complete -c pomsky -s V -l version          -d 'Print version information'
complete -c pomsky -s W -l warnings         -d 'Disable some or all warnings' -xa "(echo \"$warnings\")"
complete -c pomsky -s d -l debug            -d 'Show debug information'
complete -c pomsky      -l json             -d 'Return output as JSON'

# test args
complete -c pomsky -n "__fish_seen_subcommand_from test" -s e -l engine             -d 'Regex engine for unit tests' -xa "(echo \"$engines\")"
complete -c pomsky -n "__fish_seen_subcommand_from test"      -l pass-with-no-tests -d 'Succeed if path contains no *.pomsky files'

# non-test args
complete -c pomsky -n "not __fish_seen_subcommand_from test"      -l list        -d 'List shorthands' -xa "shorthands"
complete -c pomsky -n "not __fish_seen_subcommand_from test" -s n -l no-new-line -d "Don't print line break after the output"
complete -c pomsky -n "not __fish_seen_subcommand_from test"      -l test        -d 'Run unit tests' -xa "(echo \"$engines\")"