#!/usr/bin/env python3

import sys
import os
import os.path as path
import glob

def main():
    project_root =  path.abspath(path.join(__file__ , "..", ".."))
    instructions_dir = path.join(project_root, "tests", "fixtures", "instructions")

    files = []
    for ext in ["o", "elf"]:
        files.extend(glob.glob(path.join(instructions_dir, "**", "*." + ext)))

    if len(files) == 0:
        print("nothing to remove")
        return

    if len(sys.argv) < 2 or sys.argv[1] != "-y":
        response = input("Delete " + files[0] + " and " + str(len(files) - 1) + " others? [y/N]: ")
        if response.lower() != "y":
            print("cancelled")
            return

    for filepath in files:
        os.remove(filepath)

    print("removed " + str(len(files)) + " files")

main()
