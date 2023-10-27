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
    nb_tests=$((nb_tests + 1))

    # run file
    $test
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        nb_success=$((nb_success + 1))
    elif [ $exit_code -eq 4 ]; then
        # not executable found, not need to try other tests
        exit 1
    else
        echo "$test failed with exit code $exit_code"
    fi
done;

echo -e "\nRan $nb_tests tests ($((nb_tests - nb_success)) Failed)"
