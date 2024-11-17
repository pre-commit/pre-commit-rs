import os
import sys


def main() -> int:
    exe = os.path.join(os.path.dirname(sys.executable), 'pre-commit-rs')
    cmd = (exe, *sys.argv[1:])
    if sys.platform == 'win32':
        import subprocess
        return subprocess.call(cmd)
    else:
        os.execvp(cmd[0], cmd)
        return -1


if __name__ == '__main__':
    raise SystemExit(main())
