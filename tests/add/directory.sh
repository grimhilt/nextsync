#!/bin/sh

source ./utils.sh

nb_tests=0
TEST_SUITE_NAME="add/directory/"

add_test_no_env() {
    touch $2
    $exe add $3
    status_cmp "$1" "$4"
}

add_test() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    add_test_no_env "$1" "$2" "$3" "$4"
}

add_dir() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir dir
    $exe add "dir" 
    res=$($exe status --nostyle)
    status_cmp "dir" "new: dir"
}

add_subdir() {
    nb_tests=$((nb_tests + 1))
    setup_env
    $exe init
    mkdir foo foo/bar
    $exe add "foo" 
    res=$($exe status --nostyle)
    status_cmp "dir" "new: foo/bar\nnew: foo"
}

add_dir
add_subdir

echo $nb_tests
exit 0
