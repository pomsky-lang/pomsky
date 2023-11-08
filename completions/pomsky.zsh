#compdef pomsky

_pomsky_complete_features() {
  _values -s , 'features' ascii-mode atomic-groups boundaries dot grapheme lazy-mode lookahead lookbehind named-groups numbered-groups ranges recursion references regexes variables
}

_pomsky_complete_lists() {
  lists=(
    'shorthands:Unicode properties and shorthands'
  )
  _describe -t lists 'lists' lists
}

_pomsky_complete_engine() {
  engine=(
    'pcre2:The PCRE2 regex engine'
  )
  _describe -t engine 'engine' engine
}

_pomsky_complete_flavor() {
  flavors=(
    'pcre:PCRE flavor'
    'python:Python re flavor'
    'java:Java flavor'
    'js:JavaScript (ECMAScript) flavor'
    'dotnet:C# (.NET) flavor'
    'ruby:Ruby (oniguruma) flavor'
    'rust:Rust regex flavor'
  )
  _describe -t flavors 'flavors' flavors
}

_pomsky_complete_path() {
  _path_files -f
}

_pomsky_complete_warnings() {
  warnings=(0 compat=0 deprecated=0)
  _describe -t warnings 'warnings' warnings
}

_pomsky() {
  local curcontext="$curcontext"

  _arguments -s -w -C \
    '(--allowed-features)--allowed-features=[Allowed features, comma-separated]: :->features' \
    '(-f --flavor)'{-f+,--flavor=}'[Regex flavor]: :->flavor' \
    '(-h --help)'{-h+,--help=}'[Show help information]' \
    '(--list)--list=[List shorthands]: :->lists' \
    '(-n --no-new-line)'{-n,--no-new-line}"[Don't print line break after the output]" \
    '(-p --path)'{-p+,--path=}'[File to compile]: :->path' \
    '(-test)--test=[Run unit tests]: :->engine' \
    '(-V --version)'{-V,--version}'[Print version information]' \
    '(-W --warnings)'{-W+,--warnings=}'[Disable some or all warnings]: :->warnings' \
    '(-d --debug)'{-d,--debug}'[Show debug information]' \
    '(--json)--json[Return output as JSON]'

  case $state in
    (none) ;;
    (features) _pomsky_complete_features ;;
    (flavor) _pomsky_complete_flavor ;;
    (path) _pomsky_complete_path ;;
    (warnings) _pomsky_complete_warnings ;;
    (lists) _pomsky_complete_lists ;;
    (engine) _pomsky_complete_engine ;;
    (*) ;;
  esac
}

compdef _pomsky pomsky