#compdef pomsky

_pomsky_cmds() {
  local commands=(
    'test:Run unit test suite'
  )
  _describe -t commands 'commands' commands
}

_pomsky_complete_features() {
  _values -s , 'features' ascii-mode atomic-groups boundaries dot grapheme intersection lazy-mode lookahead lookbehind named-groups numbered-groups ranges recursion references regexes variables
}

_pomsky_complete_lists() {
  local lists=(
    'shorthands:Unicode properties and shorthands'
  )
  _describe -t lists 'lists' lists
}

_pomsky_complete_engine() {
  local engine=(
    'pcre2:PCRE2 regex engine'
    'rust:Rust crate `regex`'
  )
  _describe -t engine 'engine' engine
}

_pomsky_complete_flavor() {
  local flavors=(
    'pcre:PCRE flavor'
    'python:Python re flavor'
    'java:Java flavor'
    'js:JavaScript (ECMAScript) flavor'
    'dotnet:C# (.NET) flavor'
    'ruby:Ruby (oniguruma) flavor'
    'rust:Rust regex flavor'
    're2:RE2 flavor'
  )
  _describe -t flavors 'flavors' flavors
}

_pomsky_complete_path() {
  _path_files -f
}

_pomsky_complete_warnings() {
  local warnings=(
    '0:Disable all warnings'
    'compat=0:Disable compatibility warnings'
    'deprecated=0:Disable deprecation warnings'
  )
  _describe -t warnings 'warnings' warnings
}

_pomsky() {
  local curcontext="$curcontext"

  local global_args=(
    '(--allowed-features)--allowed-features=[Allowed features, comma-separated]: :->features'
    '(-f --flavor)'{-f+,--flavor=}'[Regex flavor]: :->flavor'
    '(-h --help)'{-h,--help}'[Show help information]'
    '(-p --path)'{-p+,--path=}'[File or directory]: :->path'
    '(-V --version)'{-V,--version}'[Print version information]'
    '(-W --warnings)'{-W+,--warnings=}'[Disable some or all warnings]: :->warnings'
    '(-d --debug)'{-d,--debug}'[Show debug information]'
    '(--json)--json[Return output as JSON]'
  )
  local test_args=(
    '(-e --engine)'{-e+,--engine=}'[Regex engine for unit tests]: :->engine'
    '(--pass-with-no-tests)--pass-with-no-tests[Succeed if path contains no *.pomsky files]'
  )
  local non_test_args=(
    '(--list)--list=[List shorthands]: :->lists'
    '(-n --no-new-line)'{-n,--no-new-line}"[Don't print line break after the output]"
    '(--test)--test=[Run unit tests]: :->engine'
  )

  # emit everything, because I haven't figured out how to do it properly
  _arguments -s -w -C '1: :_pomsky_cmds' $global_args $non_test_args $test_args

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