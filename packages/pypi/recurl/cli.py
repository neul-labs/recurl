#!/usr/bin/env python3
"""
CLI entry points for the recurl Python wrapper.
"""

import sys

from recurl import run, run_daemon


def main() -> None:
    sys.exit(run())


def main_daemon() -> None:
    sys.exit(run_daemon())


if __name__ == "__main__":
    main()
