_complete_pomsky()
{
    local flavors="pcre python java js dotnet ruby rust"
    local flavors_concat="-fpcre -fpython -fjava -fjs -fdotnet -fruby -frust"
    local warnings="0 compat=0 deprecated=0"
    local warnings_concat="-W0 -Wcompat=0 -Wdeprecated=0"
    local features="atomic-groups boundaries dot grapheme lazy-mode lookahead lookbehind named-groups numbered-groups ranges references regexes variables"
    local flags="--allowed-features --flavor --help --no-new-line --path --version --warnings --debug --json --list"

    local cur=${COMP_WORDS[COMP_CWORD]}
    local prev=${COMP_WORDS[COMP_CWORD - 1]}

    case "$prev" in
        -p | --path)
            COMPREPLY=( $( compgen -o plusdirs -f -- $cur ) )

            # add '/' after directories and a space after files
            for ((i=0; i < ${#COMPREPLY[@]}; i++)); do
                if [ -d "${COMPREPLY[$i]}" ]; then
                    COMPREPLY[$i]="${COMPREPLY[$i]}/"
                else
                    COMPREPLY[$i]="${COMPREPLY[$i]} "
                fi
            done
            return 0;
            ;;
        -f | --flavor)
            COMPREPLY=( $( compgen -W "$flavors" -- $cur ) )
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
        *)
            if [[ $cur = -f* ]]; then
                COMPREPLY=( $( compgen -W "$flavors_concat" -- $cur ) )
            elif [[ $cur = -W* ]]; then
                COMPREPLY=( $( compgen -W "$warnings_concat" -- $cur ) )
            else
                COMPREPLY=( $( compgen -W "$flags" -- $cur ) )
            fi
            ;;
    esac

    # add a space after each completion
    for ((i=0; i < ${#COMPREPLY[@]}; i++)); do
        COMPREPLY[$i]="${COMPREPLY[$i]} "
    done
}
complete -o nospace -F _complete_pomsky pomsky
