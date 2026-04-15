#!/usr/bin/env python3

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate latest.json for the Tauri updater.")
    parser.add_argument("--version", required=True)
    parser.add_argument("--notes", default="")
    parser.add_argument("--linux-url", required=True)
    parser.add_argument("--linux-sig-file", required=True)
    parser.add_argument("--windows-url", required=True)
    parser.add_argument("--windows-sig-file", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    payload = {
        "version": args.version,
        "pub_date": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "platforms": {
            "linux-x86_64": {
                "signature": Path(args.linux_sig_file).read_text(encoding="utf-8").strip(),
                "url": args.linux_url,
            },
            "windows-x86_64": {
                "signature": Path(args.windows_sig_file).read_text(encoding="utf-8").strip(),
                "url": args.windows_url,
            },
        },
    }

    if args.notes.strip():
        payload["notes"] = args.notes.strip()

    Path(args.output).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
