#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import re
import textwrap


def formula_class(name: str) -> str:
    parts = re.split(r"[^A-Za-z0-9]+", name)
    return "".join(part[:1].upper() + part[1:] for part in parts if part)


def render_formula(
    *,
    name: str,
    description: str,
    homepage: str,
    url: str,
    sha256: str,
    bin_name: str,
) -> str:
    klass = formula_class(name)
    return textwrap.dedent(
        f"""\
        class {klass} < Formula
          desc "{description}"
          homepage "{homepage}"
          url "{url}"
          sha256 "{sha256}"
          license "AGPL-3.0-only"

          livecheck do
            url :stable
            regex(/^v?(\\d+(?:\\.\\d+)+)$/i)
          end

          depends_on "rust" => :build

          def install
            system "cargo", "install", *std_cargo_args
          end

          test do
            (testpath/"input.json").write <<~JSON
              {{"count":3,"items":[1,2,3],"ready":true}}
            JSON

            output = shell_output("#{{bin}}/{bin_name} emit #{{testpath/"input.json"}} --backend typst")
            assert_match "#let input", output
            assert_match "\\"count\\": 3", output
            assert_match "\\"ready\\": true", output
          end
        end
        """
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--name", required=True)
    parser.add_argument("--description", required=True)
    parser.add_argument("--homepage", required=True)
    parser.add_argument("--url", required=True)
    parser.add_argument("--sha256", required=True)
    parser.add_argument("--bin", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output = pathlib.Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        render_formula(
            name=args.name,
            description=args.description,
            homepage=args.homepage,
            url=args.url,
            sha256=args.sha256,
            bin_name=args.bin,
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
