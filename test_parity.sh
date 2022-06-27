#!/bin/sh
cargo build --release
OINK_PATH="../oink/build/oink"
TEST_PATH="./inputs/tests/*"
EXEC_PATH="./target/release/lmc"

retval=0
for t in $TEST_PATH
do
        oink_result=$($OINK_PATH -p --no $t | grep -o -E 'won by.*')
        lmc_result=$($EXEC_PATH parity -r $t)
        if [[ $oink_result = $lmc_result ]]
        then
                echo "okay $t"
        else
                echo $(echo "fail $t: oink: \"$oink_result\"" | sed "s/'\n'/'\\n'/")
                echo $(echo "fail $t: lmc : \"$lmc_result\"" | sed "s/'\n'/'\\n'/")
                retval=1
        fi
done

exit $retval
