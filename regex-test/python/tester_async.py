import sys
import re
import fileinput

try:
    for line in fileinput.input():
        try:
            re.compile(line)
            print("success")
        except Exception as e:
            print(str(e).replace("\\", "\\\\").replace("\n", "\\n"))
        sys.stdout.flush()
except KeyboardInterrupt:
    pass