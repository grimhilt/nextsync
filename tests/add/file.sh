#!/bin/sh

get_exe() {
    exe=$(pwd)
    exe+="/../target/debug/nextsync"
    if [ ! -f $exe ]; then
        echo "No executable found, try to compile first" 
        exit 4
    fi
}
setup_env() {
    [ ! -v exe ] && get_exe
    path=$(mktemp -d)
    cd $path
}

add_cmp() {
    res=$($exe status --nostyle)
    diff  <(echo -e "$2" ) <(echo -e "$res") 2> /dev/null > /dev/null
    if [ $? -ne 0 ]; then
        echo -e "$1: Output differ:"
        diff  <(echo -e "$2" ) <(echo -e "$res")
        echo $path
        exit 1
    fi
}

add_test() {
    setup_env
    $exe init
    touch $2
    $exe add $3
    add_cmp "$1" "$4"
}

add_basics() {
    add_test "basic" "toto" "toto" "new: toto"
}

add_space() {
    setup_env
    $exe init
    touch 'to to' 
    $exe add 'to to' 
    res=$($exe status --nostyle)
    add_cmp "space" "new: to to"
}

add_multiple() {
    add_test "multiple" "titi riri" "titi riri" "new: titi\nnew: riri"
}

add_regex() {
    add_test "regex" "titi riri" "./*" "new: riri\nnew: titi"
}

add_subdir() {
    setup_env
    $exe init
    mkdir dir
    touch dir/toto 
    $exe add "./dir/toto" 
    res=$($exe status --nostyle)
    add_cmp "subdir" "new: dir/toto"
}

add_subdir_regex() {
    setup_env
    $exe init
    mkdir dir
    touch dir/toto dir/roro 
    $exe add "./dir/*" 
    res=$($exe status --nostyle)
    add_cmp "subdir_regex" "new: dir/roro\nnew: dir/toto"
}

add_basics
add_space
add_multiple
add_regex
add_subdir
#add_subdir_regex

exit 0
