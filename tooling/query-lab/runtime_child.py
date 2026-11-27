"""Apply resource limits to an explicitly requested fixed-example process."""
import os
import resource
import sys

MAX_CPU_SECONDS = 20
MAX_FILE_BYTES = 64 * 1024 * 1024
MAX_DESCRIPTORS = 256

for kind, requested in ((resource.RLIMIT_CPU, MAX_CPU_SECONDS),
                        (resource.RLIMIT_FSIZE, MAX_FILE_BYTES),
                        (resource.RLIMIT_NOFILE, MAX_DESCRIPTORS),
                        (resource.RLIMIT_CORE, 0)):
    _, hard = resource.getrlimit(kind)
    limit = min(requested, hard) if hard != resource.RLIM_INFINITY else requested
    resource.setrlimit(kind, (limit, limit))
os.execvp(sys.argv[1], sys.argv[1:])
