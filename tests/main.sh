#!/bin/sh

# Getting all tests
TESTS=$(find -name "*.sh" -not -name "main.sh")
if [ $# -ne 0 ]; then
    TESTS=$(find -name "*$1*" -not -name "main.sh") 
    tests=""
    for obj in $TESTS; do
        [ -d $obj ] && tests+=$(find -path "$obj/*.sh" -not -name "main.sh")
    done;
    TESTS=$tests
fi

# Executing tests
nb_tests=0
nb_success=0
for test in $TESTS; do
    #nb_tests=$((nb_tests + 1))

    # run file
    tmp_stderr=$(mktemp)
    nb_tests_tmp=$($test 2>"$tmp_stderr")
    exit_code=$?
    capture_stderr=$(<"$tmp_stderr")
    [ "$capture_stderr" != "" ] && echo -e "$capture_stderr"
    rm $tmp_stderr
   
    # add nb_tests from executed test_suite to global nb_test
    [ "$nb_tests_tmp" != "" ] &&
        [ $nb_tests_tmp -gt 0 ] &&
        nb_tests=$((nb_tests + nb_tests_tmp))

    if [ $exit_code -eq 0 ]; then
        nb_success=$((nb_success + nb_tests_tmp))
    elif [ $exit_code -eq 4 ]; then
        # not executable (nextsync) found, not need to try other tests
        exit 1
    else
        nb_success=$((nb_success + nb_tests_tmp - 1))
        echo "$test failed with exit code $exit_code"
    fi
done;

echo -e "\nRan $nb_tests tests ($((nb_tests - nb_success)) Failed)"
