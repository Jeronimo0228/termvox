#!/usr/bin/env python3
"""Minimal TermVox JSON-RPC v1 plugin for documentation and conformance tests."""

import json
import sys


def respond(request: dict) -> dict:
    method = request.get("method")
    if method == "initialize":
        result = {
            "id": "example",
            "name": "Example Agent",
            "version": "0.1.0",
            "protocol_version": 1,
            "capabilities": {
                "streaming": False,
                "resume": False,
                "cancellation": False,
            },
        }
    elif method == "probe":
        result = {"ok": True, "detail": "example plugin is ready"}
    elif method == "start":
        result = {"session_id": "example-session"}
    elif method == "send":
        params = request.get("params", {})
        result = {
            "session_id": params.get("session_id"),
            "text": params.get("prompt", ""),
        }
    elif method in {"cancel", "shutdown"}:
        # The alpha client currently expects a non-null result.
        result = True
    else:
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "error": {"code": -32601, "message": "method not found"},
        }
    return {"jsonrpc": "2.0", "id": request.get("id"), "result": result}


for line in sys.stdin:
    try:
        print(json.dumps(respond(json.loads(line))), flush=True)
    except Exception as error:  # Protocol harness must always return JSON.
        print(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32603, "message": str(error)},
                }
            ),
            flush=True,
        )
