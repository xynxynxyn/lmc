#!/usr/bin/python3
import glob
import subprocess

OINK_PATH = "../oink/build/oink"
TEST_PATH = "./inputs/tests/*"
EXEC_PATH = "./target/release/lmc"


def sh(cmd):
    ps = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE)
    return {"text": ps.communicate()[0], "status": ps.returncode}


def test_fpi(file):
    lmc_regions = sh(f"{EXEC_PATH} parity -r {file}")["text"]
    oink_regions = sh(
        f"{OINK_PATH} -p --no {file} | grep -o -E 'won by.*'")["text"]

    if lmc_regions != oink_regions:
        raise AssertionError(
            f"winning regions differ:\n\toink: {oink_regions}\n\tlmc:  {lmc_regions}"
        )

    sh(f"{EXEC_PATH} parity -s {file} > game.sol")
    oink_verify = sh(f"{OINK_PATH} -v {file} --sol game.sol")
    sh("rm game.sol")

    if oink_verify["status"] != 0:
        raise AssertionError(f"oink could not verify solution")


if __name__ == "__main__":
    for file in sorted(glob.glob(TEST_PATH)):
        fpi = " OK"
        try:
            test_fpi(file)
        except AssertionError as error:
            print(f"{file} {error}")
            fpi = "ERR"

        print("file {}: fpi {}".format(file, fpi))
