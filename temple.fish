set -l commands list new init deinit help create remove info 'debug-config'
set -l templates (temple list -se 2> /dev/null | tr " " "\n" || echo "")

function __fish_temple_contains_temple_new
    set -l cmd_args (commandline -opc)
    set -l list_arr (temple list -se 2> /dev/null | tr " " "\n" || echo "")
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
        temple list -spe 2> /dev/null | tr " " "\n" || echo ""
    end
end

function __fish_temple_help_subcommand_completion
    set -l commands list new init deinit help create remove info 'debug-config'
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
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a info -d 'Get information for a template'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a init -d 'Initialize a template config directory'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a deinit -d 'Remove a temple config directory'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a create -d 'Create a new empty template. You can then place files in it'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a remove -d 'Remove an existing template'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a debug-config -d 'Parse and dump objects to stdout'
complete -c temple -n "not __fish_seen_subcommand_from $commands" -a help -d 'Print this message or the help of the given subcommand(s)'

# init
complete -c temple -n "__fish_seen_subcommand_from init; and __fish_seen_subcommand_from local" -l not-hidden -d 'Name the local folder "temple" instead of ".temple"'
complete -c temple -n "__fish_seen_subcommand_from init; and not __fish_seen_subcommand_from global local" -a global -d 'Create the global temple configuration dir'
complete -c temple -n "__fish_seen_subcommand_from init; and not __fish_seen_subcommand_from global local" -a local -d 'Create a new temple local configuration dir'

# deinit
complete -c temple -n "__fish_seen_subcommand_from deinit; and not __fish_seen_subcommand_from global local" -a global -d 'Create the global temple configuration dir'
complete -c temple -n "__fish_seen_subcommand_from deinit; and not __fish_seen_subcommand_from global local" -a local -d 'Create a new temple local configuration dir in the current dir'

# create
complete -c temple -n "__fish_seen_subcommand_from create; and not __fish_seen_subcommand_from global local" -a global -d 'Create a new empty global template'
complete -c temple -n "__fish_seen_subcommand_from create; and not __fish_seen_subcommand_from global local" -a local -d 'Create a new empty local template'
complete -c temple -n "__fish_seen_subcommand_from create; and __fish_seen_subcommand_from global local" -F

# remove
complete -c temple -n "__fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from help" -ka '(__fish_temple_complete_templates)'

# debug
complete -c temple -n "__fish_seen_subcommand_from debug-config" -F

# help
complete -c temple -f -n "__fish_seen_subcommand_from help" -a "(__fish_temple_help_subcommand_completion)"

# list
complete -c temple -n "__fish_seen_subcommand_from list" -s s -l short -d 'Show templates in a single space separated list'
complete -c temple -n "__fish_seen_subcommand_from list" -s p -l path -d 'Show templates path'

# new
complete -c temple -n "__fish_seen_subcommand_from new info; and not __fish_seen_subcommand_from help" -ka '(__fish_temple_complete_templates)'
# complete -c temple -n "__fish_seen_subcommand_from new info; and __fish_seen_subcommand_from $templates" -n "not contains -- -- (commandline -opc)" -a '(__fish_temple_c_complete)' 
