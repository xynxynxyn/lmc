#!/usr/bin/python3
import glob
import os
import sys
import subprocess

OINK_PATH = "../oink/build/oink"
TEST_DIR = "./inputs/tests"
EXEC_PATH = "./target/release/lmc"


def sh(cmd):
    ps = subprocess.Popen(cmd, shell=True, stdout=subprocess.PIPE)
    return {"text": ps.communicate()[0], "status": ps.returncode}


def test_generic(file, algorithm):
    lmc_regions = sh(f"{EXEC_PATH} parity -a {algorithm} -r {file}")["text"]
    oink_regions = sh(
        f"{OINK_PATH} -p --no {file} | grep -o -E 'won by.*'")["text"]

    if lmc_regions != oink_regions:
        raise AssertionError(
            f"winning regions differ:\n\toink: {oink_regions}\n\tlmc:  {lmc_regions}"
        )

    sh(f"{EXEC_PATH} parity -s -a {algorithm} {file} > game.sol")
    oink_verify = sh(f"{OINK_PATH} -v {file} --sol game.sol")
    sh("rm game.sol")

    if oink_verify["status"] != 0:
        raise AssertionError(f"oink could not verify solution")


def test_fpi(file):
    test_generic(file, "fpi")


def test_zielonka(file):
    test_generic(file, "zielonka")


if __name__ == "__main__":
    print(f"compiling executable")
    sh("cargo build --release")

    if not os.path.exists(EXEC_PATH):
        print(f"ERR could not find executable {EXEC_PATH}")
        sys.exit(1)

    if not os.path.exists(OINK_PATH):
        print(f"ERR could not find oink executable {OINK_PATH}")
        sys.exit(1)

    if not os.path.isdir(TEST_DIR):
        print(f"ERR could not find test directory {TEST_DIR}")
        sys.exit(1)

    for file in sorted(glob.glob(f"{TEST_DIR}/*")):
        fpi = " OK"
        try:
            test_fpi(file)
        except AssertionError as error:
            print(f"{file} {error}")
            fpi = "ERR"

        zielonka = " OK"
        try:
            test_zielonka(file)
        except AssertionError as error:
            print(f"{file} {error}")
            zielonka = "ERR"

        print("file {}: fpi {}   zielonka {}".format(file, fpi, zielonka))
