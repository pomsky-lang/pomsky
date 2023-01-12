import sys
import re

try:
    re.compile(sys.argv[1])
except Exception as e:
    print(e)
    sys.exit(1)
