set -l features \
'ascii-mode atomic-groups boundaries dot grapheme lazy-mode lookahead lookbehind named-groups numbered-groups ranges references regexes variables'
set -l flavors \
'pcre	PCRE flavor
python	Python re flavor
java	Java flavor
js		JavaScript (ECMAScript) flavor
dotnet	C# (.NET) flavor
ruby	Ruby (Oniguruma) flavor
rust	Rust regex flavor'
set -l warnings \
'0				Disable all warnings
compat=0		Disable compatibility warnings
deprecated=0	Disable deprecation warnings'

complete -c pomsky      -l allowed-features -d 'Allowed features, comma-separated' -xa "(__fish_append , $features)"
complete -c pomsky -s f -l flavor           -d 'Regex flavor' -xa "(echo \"$flavors\")"
complete -c pomsky -s h -l help             -d 'Show help information'
complete -c pomsky      -l list             -d 'List shorthands' -xa "shorthands"
complete -c pomsky -s n -l no-new-line      -d "Don't print line break after the output"
complete -c pomsky -s p -l path             -d 'File to compile' -kxa "(__fish_complete_suffix .pom)"
complete -c pomsky -s V -l version          -d 'Print version information'
complete -c pomsky -s W -l warnings         -d 'Disable some or all warnings' -xa "(echo \"$warnings\")"
complete -c pomsky -s d -l debug            -d 'Show debug information'
complete -c pomsky      -l json             -d 'Return output as JSON'
