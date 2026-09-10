#!/usr/bin/env python3
"""
cybercore_api.py — a zero-dependency HTTP API over the CYBERGRID schema.

Serves the single source of truth from ``cybercore/schema/cybergrid.json``
(the CYBERGRID palette + canonical filesystem paths) as JSON over HTTP, so
consumers that can't embed the Rust crate or read the file directly —
shell scripts, remote hosts, a browser dev console, QML prototypes — get
the same values over ``curl`` instead of keeping their own drifting copy.

Standard library only. Python 3.8+.

    python3 cybercore_api.py                       # http://127.0.0.1:8787
    python3 cybercore_api.py --port 9000
    python3 cybercore_api.py --host 0.0.0.0        # expose on the LAN
    python3 cybercore_api.py --schema ./schema/cybergrid.json

Endpoints (all GET, all JSON):

    /                     service metadata + this endpoint list
    /health               {"status": "ok"}
    /version              {"schema_version": <int>}
    /schema               the full cybergrid.json, byte-for-byte
    /palette              every colour; ?format= applies to all of them
    /palette/<name>       one colour; ?format=hex|css|rgb|ansi|ansi-bg|all
    /paths                every path; ?expand=true expands a leading "~/"
    /paths/<name>         one path; ?expand=true

Colour formats:

    hex      c8e967                    as stored in the schema, no "#"
    css      #c8e967
    rgb      rgb(200, 233, 103)
    ansi     \\x1b[38;2;200;233;103m    24-bit foreground SGR escape
    ansi-bg  \\x1b[48;2;200;233;103m    24-bit background SGR escape
    all      object carrying every representation above

The schema is looked up in this order (first hit wins):

    1. --schema PATH
    2. $CYBERGRID_SCHEMA
    3. <dir of this script>/schema/cybergrid.json
    4. ~/.sysops/cybercore/schema/cybergrid.json
    5. a copy embedded in this file (keeps it self-contained)

The file is re-read when its mtime changes, so editing the JSON is picked
up without restarting the server.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

__version__ = "1.0.0"

# --- embedded fallback (verbatim copy of schema/cybergrid.json) ---------------
EMBEDDED_SCHEMA_JSON = """{
  "schema_version": 1,
  "palette": {
    "bg": "0e091d",
    "acid_green": "c8e967",
    "hot_pink": "FD3E6A",
    "purple": "9147a8",
    "cyan": "14B9B5",
    "orange": "FF7F41",
    "red": "f93d3b",
    "white": "ffffff"
  },
  "paths": {
    "sysops_root": "~/.sysops",
    "cargo_target_root": "~/.cargo-target",
    "arch_sys_root": "~/.arch-sys",
    "omarchy_config_root": "~/.config/omarchy"
  }
}
"""

COLOR_FORMATS = ("hex", "css", "rgb", "ansi", "ansi-bg", "all")
TRUE_VALUES = {"1", "true", "yes", "on"}


# --- schema loading ---------------------------------------------------------


def candidate_paths(explicit: "str | None") -> "list[str]":
    here = os.path.dirname(os.path.abspath(__file__))
    out = []
    if explicit:
        out.append(explicit)
    env = os.environ.get("CYBERGRID_SCHEMA")
    if env:
        out.append(env)
    out.append(os.path.join(here, "schema", "cybergrid.json"))
    out.append(os.path.expanduser("~/.sysops/cybercore/schema/cybergrid.json"))
    return out


class Schema:
    """Parsed schema plus the raw bytes it came from, cached by mtime."""

    def __init__(self, explicit_path: "str | None"):
        self._explicit = explicit_path
        self._path: "str | None" = None
        self._mtime: "float | None" = None
        self.raw: str = ""
        self.data: dict = {}
        self.source: str = ""
        self.reload()

    def reload(self) -> None:
        for path in candidate_paths(self._explicit):
            try:
                st = os.stat(path)
            except OSError:
                continue
            if path == self._path and st.st_mtime == self._mtime and self.data:
                return  # unchanged
            try:
                with open(path, "r", encoding="utf-8") as fh:
                    raw = fh.read()
                data = json.loads(raw)
            except (OSError, ValueError) as exc:
                print(f"cybercore_api: {path}: {exc}; trying next", file=sys.stderr)
                continue
            self._path, self._mtime = path, st.st_mtime
            self.raw, self.data, self.source = raw, data, path
            return
        # nothing on disk — fall back to the embedded copy
        if self.source != "embedded":
            print("cybercore_api: no schema file found, using embedded copy", file=sys.stderr)
        self._path = self._mtime = None
        self.raw = EMBEDDED_SCHEMA_JSON
        self.data = json.loads(EMBEDDED_SCHEMA_JSON)
        self.source = "embedded"


# --- colour + path helpers ------------------------------------------------


def hex_to_rgb(value: str) -> "tuple[int, int, int]":
    h = value.lstrip("#").strip()
    if len(h) != 6 or any(c not in "0123456789abcdefABCDEF" for c in h):
        raise ValueError(f"not a 6-digit hex colour: {value!r}")
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def format_color(value: str, fmt: str):
    h = value.lstrip("#").strip().lower()
    if fmt == "hex":
        return h
    if fmt == "css":
        return "#" + h
    r, g, b = hex_to_rgb(h)
    if fmt == "rgb":
        return f"rgb({r}, {g}, {b})"
    if fmt == "ansi":
        return f"\x1b[38;2;{r};{g};{b}m"
    if fmt == "ansi-bg":
        return f"\x1b[48;2;{r};{g};{b}m"
    if fmt == "all":
        return {
            "hex": h,
            "css": "#" + h,
            "rgb": f"rgb({r}, {g}, {b})",
            "rgb_tuple": [r, g, b],
            "ansi": f"\x1b[38;2;{r};{g};{b}m",
            "ansi_bg": f"\x1b[48;2;{r};{g};{b}m",
        }
    raise ValueError(fmt)


def expand_home(path: str) -> str:
    if path.startswith("~/") or path == "~":
        return os.path.expanduser(path)
    return path


# --- HTTP handler -------------------------------------------------------------


class Handler(BaseHTTPRequestHandler):
    server_version = f"cybercore_api/{__version__}"
    schema: Schema  # injected on the server instance

    # -- plumbing --

    def _send(self, status, obj=None, raw=None):
        if raw is None:
            raw = json.dumps(obj, indent=2, ensure_ascii=False) + "\n"
        body = raw.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, OPTIONS")
        self.send_header("X-Cybergrid-Source", self.server.schema.source)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def _error(self, status, message, **extra):
        payload = {"error": message}
        payload.update(extra)
        self._send(status, payload)

    def log_message(self, fmt, *args):
        sys.stderr.write(
            "%s  %s\n" % (self.log_date_time_string(), fmt % args)
        )

    def do_OPTIONS(self):  # noqa: N802
        self.send_response(HTTPStatus.NO_CONTENT)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.end_headers()

    def do_HEAD(self):  # noqa: N802
        self.do_GET()

    # -- routing --

    def do_GET(self):  # noqa: N802
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p != ""]
        query = parse_qs(parsed.query)

        self.server.schema.reload()
        schema = self.server.schema

        try:
            if not parts:
                return self._route_index()
            if parts == ["health"]:
                return self._send(HTTPStatus.OK, {"status": "ok"})
            if parts == ["version"]:
                return self._send(
                    HTTPStatus.OK,
                    {
                        "schema_version": schema.data.get("schema_version"),
                        "api_version": __version__,
                        "source": schema.source,
                    },
                )
            if parts == ["schema"]:
                return self._send(HTTPStatus.OK, raw=schema.raw if schema.raw.endswith("\n") else schema.raw + "\n")
            if parts[0] == "palette":
                return self._route_palette(parts[1:], query)
            if parts[0] == "paths":
                return self._route_paths(parts[1:], query)
        except ValueError as exc:
            return self._error(HTTPStatus.BAD_REQUEST, str(exc))
        except Exception as exc:  # pragma: no cover - defensive
            return self._error(HTTPStatus.INTERNAL_SERVER_ERROR, f"{type(exc).__name__}: {exc}")

        return self._error(
            HTTPStatus.NOT_FOUND,
            f"no such endpoint: /{'/'.join(parts)}",
            endpoints=self._endpoint_list(),
        )

    def do_POST(self):  # noqa: N802
        self._reject_method()

    do_PUT = do_DELETE = do_PATCH = do_POST

    def _reject_method(self):
        self._error(
            HTTPStatus.METHOD_NOT_ALLOWED,
            f"{self.command} not allowed; this API is read-only (GET)",
        )

    # -- route handlers --

    def _endpoint_list(self):
        return [
            "/", "/health", "/version", "/schema",
            "/palette", "/palette/<name>?format=hex|css|rgb|ansi|ansi-bg|all",
            "/paths", "/paths/<name>?expand=true",
        ]

    def _route_index(self):
        schema = self.server.schema
        self._send(
            HTTPStatus.OK,
            {
                "service": "cybercore-api",
                "description": "HTTP view of the CYBERGRID schema (palette + canonical paths)",
                "api_version": __version__,
                "schema_version": schema.data.get("schema_version"),
                "schema_source": schema.source,
                "endpoints": self._endpoint_list(),
                "palette_names": sorted(schema.data.get("palette", {})),
                "path_names": sorted(schema.data.get("paths", {})),
            },
        )

    def _get_format(self, query):
        raw = query.get("format", ["hex"])[-1].lower()
        if raw not in COLOR_FORMATS:
            raise ValueError(
                f"unknown format {raw!r}; choose one of: {', '.join(COLOR_FORMATS)}"
            )
        return raw

    def _route_palette(self, rest, query):
        palette = self.server.schema.data.get("palette", {})
        fmt = self._get_format(query)

        if not rest:
            out = {name: format_color(val, fmt) for name, val in palette.items()}
            return self._send(HTTPStatus.OK, out)

        if len(rest) > 1:
            raise ValueError(f"not a palette entry: {'/'.join(rest)}")

        name = rest[0]
        if name not in palette:
            return self._error(
                HTTPStatus.NOT_FOUND,
                f"no such colour: {name!r}",
                available=sorted(palette),
            )
        return self._send(
            HTTPStatus.OK,
            {"name": name, "format": fmt, "value": format_color(palette[name], fmt)},
        )

    def _route_paths(self, rest, query):
        paths = self.server.schema.data.get("paths", {})
        expand = query.get("expand", ["false"])[-1].lower() in TRUE_VALUES

        if not rest:
            out = {
                name: (expand_home(val) if expand else val)
                for name, val in paths.items()
            }
            return self._send(HTTPStatus.OK, out)

        if len(rest) > 1:
            raise ValueError(f"not a path entry: {'/'.join(rest)}")

        name = rest[0]
        if name not in paths:
            return self._error(
                HTTPStatus.NOT_FOUND,
                f"no such path: {name!r}",
                available=sorted(paths),
            )
        raw_val = paths[name]
        return self._send(
            HTTPStatus.OK,
            {
                "name": name,
                "value": expand_home(raw_val) if expand else raw_val,
                "raw": raw_val,
                "expanded": expand_home(raw_val),
            },
        )


# --- entry point -----------------------------------------------------------


def main(argv=None):
    ap = argparse.ArgumentParser(
        prog="cybercore_api.py",
        description="Zero-dependency HTTP API over the CYBERGRID schema.",
    )
    ap.add_argument("--host", default="127.0.0.1", help="bind address (default: 127.0.0.1)")
    ap.add_argument("--port", type=int, default=8787, help="bind port (default: 8787)")
    ap.add_argument("--schema", metavar="PATH", help="path to cybergrid.json")
    ap.add_argument("--version", action="store_true", help="print version and exit")
    args = ap.parse_args(argv)

    if args.version:
        s = Schema(args.schema)
        print(f"cybercore_api {__version__} "
              f"(schema_version {s.data.get('schema_version')} from {s.source})")
        return 0

    schema = Schema(args.schema)
    httpd = ThreadingHTTPServer((args.host, args.port), Handler)
    httpd.schema = schema

    print(
        f"cybercore-api {__version__} listening on http://{args.host}:{args.port}\n"
        f"  schema : {schema.source} (schema_version {schema.data.get('schema_version')})\n"
        f"  try    : curl -s http://{args.host}:{args.port}/palette/acid_green?format=css",
        file=sys.stderr,
    )
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\ncybercore-api: shutting down", file=sys.stderr)
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
