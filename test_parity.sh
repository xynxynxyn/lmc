#!/bin/sh
OINK_PATH="../oink/build/oink"
TEST_PATH="./inputs/tests/*"
EXEC_PATH="./target/release/lmc"

retval=0
for t in $TEST_PATH
do
        echo "Testing " $t
        eval $OINK_PATH '-v' $t
        oink_result=$?
        lmc_result=eval $EXEC_PATH 'parity -v' $t
        lmc_result=$?
        if [ $oink_result -eq $lmc_result ]
        then
                echo "Okay " $t
        else
                echo "Fail $t Oink: $oink_result, lmc_result: $lmc_result"
                retval=1
        fi
done

exit $retval
