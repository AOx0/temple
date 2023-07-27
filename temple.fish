set -l commands list new init help
set -l templates (temple list --short 2> /dev/null | tr " " "\n" || echo "")

function __fish_temple_contains_temple_new
    set -l cmd_args (commandline -opc)
    set -l list_arr (temple list --short 2> /dev/null | tr " " "\n" || echo "")
    set -l contains "n"

    for arg in $cmd_args
        if contains -- $arg $list_arr
            set contains "s"
        end
    end

    echo $contains
end

function __fish_temple_complete_templates
    set -l contains (__fish_temple_contains_temple_new)

    if [ "$contains" = "n" ]
        temple list -sp 2> /dev/null | tr " " "\n" || echo ""
    end
end

function __fish_temple_help_subcommand_completion
    set -l commands list new init help
    set -l cmd_args (commandline -opc)

    if test (count $cmd_args) -eq 2
        echo $commands 2> /dev/null | tr " " "\n" || echo ""
    end
end

function __fish_temple_c_complete
    # get the argument to 'c'
    set arg (commandline -ct)

    # save our PWD
    set saved_pwd $PWD
    set projects $PWD

    # cd to $PROJECTS (and then back after)
    # while in $PROJECTS, complete as if we are 'cd'
    builtin cd $projects
    and complete -C"cd $arg"
    builtin cd $saved_pwd
end

# don't suggest files right off
# complete -c temple -n "__fish_is_first_arg" --no-files
complete -c temple -f

complete -c temple -n "not __fish_seen_subcommand_from help" -s h -l help -d 'Print help'

# commands
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a new -d 'Create a new project from a template'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a list -d 'List existing templates'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a init -d 'Initialize template directory at ~/.'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a help -d 'Print this message or the help of the given subcommand(s)'

# help
complete -c temple -f -n "__fish_seen_subcommand_from help" -a "(__fish_temple_help_subcommand_completion)"

# list
complete -c temple -n "__fish_seen_subcommand_from list" -s s -l short -d 'Show templates in a single space separated list'
complete -c temple -n "__fish_seen_subcommand_from list" -s p -l path -d 'Show templates path'

# new
complete -c temple -n "__fish_seen_subcommand_from new; and not __fish_seen_subcommand_from help" -ka '(__fish_temple_complete_templates)'
complete -c temple -n "__fish_seen_subcommand_from new" -s l -l local -d 'Prefer local (./.temple/template_name) if available [default: prefer ~/.temple/template_name]'
complete -c temple -n "__fish_seen_subcommand_from new" -s i -l in_place -d 'Place contents in_place (./.) instead of creating a folder'
complete -c temple -n "__fish_seen_subcommand_from new" -s o -l overwrite -d 'Overwrite any already existing files'
complete -c temple -n "__fish_seen_subcommand_from new; and __fish_seen_subcommand_from $templates" -a '(__fish_temple_c_complete)' 