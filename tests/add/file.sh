#!/bin/sh

source ./utils.sh

nb_tests=0
TEST_SUITE_NAME="add/file/"

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

add_cmp() {
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

add_test_no_env() {
    touch $2
    $exe add $3
    add_cmp "$1" "$4"
}

add_test() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    add_test_no_env "$1" "$2" "$3" "$4"
}

add_basics() {
    add_test "basic" "toto" "toto" "new: toto"
}

add_space() {
    nb_tests=$((nb_tests + 1))
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

add_file_subdir() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    touch dir/toto 
    $exe add "./dir/toto" 
    res=$($exe status --nostyle)
    add_cmp "file_subdir" "new: dir/toto"
}

add_whole_subdir() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    touch dir/toto 
    touch dir/roro 
    $exe add "dir" 
    res=$($exe status --nostyle)
    add_cmp "whole_subdir" "new: dir/roro\nnew: dir/toto\nnew: dir"
}

add_subdir_regex() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    touch dir/toto dir/roro 
    $exe add "./dir/*" 
    res=$($exe status --nostyle)
    add_cmp "subdir_regex" "new: dir/roro\nnew: dir/toto"
}

add_duplication() {
    add_test "duplication" "toto" "toto toto" "new: toto"
}

add_duplication_subdir() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    add_test_no_env "duplication_subdir" "dir/toto" "dir/toto dir/toto" "new: dir/toto"
}

add_all() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    touch dir/toto dir/roro lolo
    $exe add -A
    res=$($exe status --nostyle)
    add_cmp "all" "new: .nextsyncignore\nnew: dir/roro\nnew: dir/toto\nnew: dir\nnew: lolo"
}

#test nextsyncignore
#test inside folder
#test -A duplication
#test add file without changes

add_basics
add_space
add_multiple
add_regex
add_file_subdir
add_whole_subdir
add_subdir_regex
add_duplication
add_duplication_subdir
add_all

echo $nb_tests
exit 0
