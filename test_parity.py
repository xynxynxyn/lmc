#!/usr/bin/python3
import glob
import os
import sys
import subprocess

OINK_PATH = os.getenv("OINK_PATH", default="../oink/build/oink")
TEST_DIR = os.getenv("TEST_DIR", default="./inputs/tests")
EXEC_PATH = os.getenv("EXEC_PATH", default="./target/release/lmc")


def sh(cmd):
    ps = subprocess.Popen(cmd,
                          shell=True,
                          stdout=subprocess.PIPE,
                          stderr=subprocess.DEVNULL)
    return {"text": ps.communicate()[0], "status": ps.returncode}


def test_generic(file, algorithm):
    lmc_regions = sh(
        f"{EXEC_PATH} parity --algorithm {algorithm} --regions --target game.sol {file}"
    )["text"]
    oink_regions = sh(
        f"{OINK_PATH} -p --no {file} | grep -o -E 'won by.*'")["text"]

    if lmc_regions != oink_regions:
        raise AssertionError(
            f"winning regions differ:\n\toink: {oink_regions}\n\tlmc:  {lmc_regions}"
        )

    oink_verify = sh(f"{OINK_PATH} -v {file} --sol game.sol")
    sh("rm game.sol")

    if oink_verify["status"] != 0:
        raise AssertionError(f"oink could not verify solution")


def test_fpi(file):
    test_generic(file, "fpi")


def test_zielonka(file):
    test_generic(file, "zielonka")


def test_tangle(file):
    test_generic(file, "tangle")


def test_spm(file):
    test_generic(file, "spm")


if __name__ == "__main__":
    print(f"compiling executable")
    sh("cargo build --release")

    if not os.path.exists(EXEC_PATH):
        print(f"ERR could not find executable {EXEC_PATH}")
        print(
            "either you are not in the project directory or the EXEC_PATH environment variable is not set"
        )
        sys.exit(1)

    if not os.path.exists(OINK_PATH):
        print(f"ERR could not find oink executable {OINK_PATH}")
        print("set the environment variable OINK_PATH")
        sys.exit(1)

    if not os.path.isdir(TEST_DIR):
        print(f"ERR could not find test directory {TEST_DIR}")
        print("set the environment variable TEST_DIR")
        sys.exit(1)

    for file in sorted(glob.glob(f"{TEST_DIR}/*")):
        fpi = "OK "
        try:
            test_fpi(file)
        except AssertionError as error:
            print(f"{file} {error}")
            fpi = "ERR"

        zielonka = "OK "
        try:
            test_zielonka(file)
        except AssertionError as error:
            print(f"{file} {error}")
            zielonka = "ERR"

        tangle = "OK "
        try:
            test_tangle(file)
        except AssertionError as error:
            print(f"{file} {error}")
            tangle = "ERR"

        spm = "OK "
        try:
            test_spm(file)
        except AssertionError as error:
            print(f"{file} {error}")
            spm = "ERR"

        print("file {}: fpi {}  zlk {}  tgl {}  spm {}".format(
            file, fpi, zielonka, tangle, spm))
