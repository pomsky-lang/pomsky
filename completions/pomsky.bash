_complete_pomsky()
{
    local flavors="pcre python java js dotnet ruby rust re2"
    local flavors_concat="-fpcre -fpython -fjava -fjs -fdotnet -fruby -frust -fre2"

    local engines="pcre2 rust"
    local engines_concat="-epcre2 -erust"

    local warnings="0 compat=0 deprecated=0"
    local warnings_concat="-W0 -Wcompat=0 -Wdeprecated=0"

    local features="atomic-groups boundaries dot grapheme lazy-mode lookahead lookbehind named-groups numbered-groups ranges recursion references regexes variables"

    local flags_and_subcommands="test --allowed-features --flavor --help --no-new-line --path --test --version --warnings --debug --json --list"
    local test_flags="--allowed-features --engine --flavor --help --pass-with-no-tests --path --version --warnings --debug --json"

    local cur=${COMP_WORDS[COMP_CWORD]}
    local prev=${COMP_WORDS[COMP_CWORD - 1]}

    _add_space()
    {
        for ((i=0; i < ${#COMPREPLY[@]}; i++)); do
            COMPREPLY[$i]="${COMPREPLY[$i]} "
        done
    }

    _add_space_or_slash()
    {
        # add '/' after directories and a space after files
        for ((i=0; i < ${#COMPREPLY[@]}; i++)); do
            if [ -d "${COMPREPLY[$i]}" ]; then
                COMPREPLY[$i]="${COMPREPLY[$i]}/"
            else
                COMPREPLY[$i]="${COMPREPLY[$i]} "
            fi
        done
    }

    case "$prev" in
        -p | --path)
            COMPREPLY=( $( compgen -o plusdirs -f -- $cur ) )
            _add_space_or_slash
            return 0;
            ;;
        -f | --flavor)
            COMPREPLY=( $( compgen -W "$flavors" -- $cur ) )
            ;;
        -e | --engine)
            COMPREPLY=( $( compgen -W "$engines" -- $cur ) )
            ;;
        --list)
            COMPREPLY=( $( compgen -W "shorthands" -- $cur ) )
            ;;
        -W | --warnings)
            COMPREPLY=( $( compgen -W "$warnings" -- $cur ) )
            ;;
        --allowed-features)
            COMPREPLY=( $( compgen -W "$features" -- $cur ) )
            ;;
        --test)
            COMPREPLY=( $( compgen -W "$engines" -- $cur ) )
            ;;
        *)
            if [[ $cur = -f* ]]; then
                COMPREPLY=( $( compgen -W "$flavors_concat" -- $cur ) )
            elif [[ $cur = -e* ]]; then
                COMPREPLY=( $( compgen -W "$engines_concat" -- $cur ) )
            elif [[ $cur = -W* ]]; then
                COMPREPLY=( $( compgen -W "$warnings_concat" -- $cur ) )
            else
                for ((i=1; i < $COMP_CWORD; i++)); do
                    if [[ ${COMP_WORDS[$i]} = test ]]; then
                        COMPREPLY=( $( compgen -W "$test_flags" -- $cur ) )
                        _add_space
                        return 0;
                    fi
                done
                COMPREPLY=( $( compgen -W "$flags_and_subcommands" -- $cur ) )
            fi
            ;;
    esac

    _add_space
}
complete -o nospace -F _complete_pomsky pomsky
