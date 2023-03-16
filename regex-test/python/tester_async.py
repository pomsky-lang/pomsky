import sys
import re
import fileinput

def substituteLf(s):
    return s.replace("\\", "\\\\").replace("\n", "\\n")

regex = None

try:
    for line in fileinput.input():
        if line.endswith("\r\n"):
            line = line[:-2]
        elif line.endswith("\n"):
            line = line[:-1]

        if regex == None:
            try:
                regex = re.compile(line)
                print("success")
            except Exception as e:
                print(substituteLf(str(e)))
            sys.stdout.flush()
        elif line.startswith("TEST:"):
            test = line[5:]
            if regex.match(test) != None:
                print("test good")
            else:
                print(substituteLf("Regex '" + regex.pattern + "' does not match '" + test + "'"))
                regex = None
            sys.stdout.flush()
        else:
            regex = None
except KeyboardInterrupt:
    pass
