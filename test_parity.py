#!/usr/bin/python3
import glob
import os
import sys
import subprocess
import random

OINK_PATH = os.getenv("OINK_PATH", default="../oink/build/oink")
RNG_PATH = os.getenv("RNG_PATH", default="../oink/build/rngame")
TEST_DIR = os.getenv("TEST_DIR", default="./inputs/tests")
EXEC_PATH = os.getenv("EXEC_PATH", default="./target/release/lmc")


def generate_game(file, size):
    sh(f"{RNG_PATH} {size} {size + random.randint(0, 1)} 1 5 > {file}")


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


def test_file(file, tangle=True, fpi=True, spm=True, zielonka=True):
    fpi_result = "OK "
    try:
        if fpi:
            test_fpi(file)
        else:
            fpi_result = "---"
    except AssertionError as error:
        print(f"{file} {error}")
        fpi_result = "ERR"

    zielonka_result = "OK "
    try:
        if zielonka:
            test_zielonka(file)
        else:
            zielonka_result = "---"
    except AssertionError as error:
        print(f"{file} {error}")
        zielonka_result = "ERR"

    tangle_result = "OK "
    try:
        if tangle:
            test_tangle(file)
        else:
            tangle_result = "---"
    except AssertionError as error:
        print(f"{file} {error}")
        tangle_result = "ERR"

    spm_result = "OK "
    try:
        if spm:
            test_spm(file)
        else:
            spm_result = "---"
    except AssertionError as error:
        print(f"{file} {error}")
        spm_result = "ERR"

    print("file {}: fpi {}  zlk {}  tgl {}  spm {}".format(
        file, fpi_result, zielonka_result, tangle_result, spm_result))


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

    if not os.path.exists(RNG_PATH):
        print(f"ERR could not find rngame executable {RNG_PATH}")
        print(
            "set the environment variable RNG_PATH or recompile oink with -DOINK_BUILD_EXTRA_TOOLS"
        )
        sys.exit(1)

    for file in sorted(glob.glob(f"{TEST_DIR}/*")):
        test_file(file)

    number_rng_games = 100
    rng_game_size = 150
    print(f"testing {number_rng_games} random large parity games")

    for i in range(0, number_rng_games):
        generate_game(f"tmp_{i}", rng_game_size)
        test_file(f"tmp_{i}", spm=False)
        sh(f"rm tmp_{i}")
