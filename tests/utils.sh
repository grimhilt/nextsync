#!/bin/sh

mktd()
{
    echo $(mktemp -d --suffix=_nextsync)
}

mktf()
{
    echo $(mktemp --suffix=_nextsync)
}

get_exe() {
    exe=$(pwd)
    exe+="/../target/debug/nextsync"
    if [ ! -f $exe ]; then
        echo "No executable found, try to compile first" >&2
        exit 4
    fi
}
setup_env() {
    [ ! -v exe ] && get_exe
    path=$(mktd)
    cd $path
}

# test_name expected_output 
status_cmp() {
    res=$($exe status --nostyle)
    diff <(echo -e "$2" ) <(echo -e "$res") 2> /dev/null > /dev/null
    if [ $? -ne 0 ]; then
        echo -e "$TEST_SUITE_NAME$1: Output differ:" >&2
        diff -u <(echo -e "$2" ) <(echo -e "$res") | grep "^[-\+\ ][^-\+]" >&2
        echo -e "\nMore in $path" >&2
        echo $nb_tests
        exit 1
    fi
}
