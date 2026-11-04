"""Exercise real stdio framing, document notifications and completion with embedded schemas."""
import json
import os
import pathlib
import select
import subprocess
import sys
import time

binary = pathlib.Path(sys.argv[1]).resolve()
error_log = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path("stdio-stderr.log")
with error_log.open("wb") as errors:
    process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=errors, bufsize=0)
    buffer = bytearray()

    def send(method, params=None, request_id=None):
        message = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            message["params"] = params
        if request_id is not None:
            message["id"] = request_id
        body = json.dumps(message).encode()
        process.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
        process.stdin.flush()

    def receive(predicate):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            split = buffer.find(b"\r\n\r\n")
            if split >= 0:
                header = bytes(buffer[:split]).decode()
                length = int(next(line.split(":", 1)[1] for line in header.split("\r\n")
                                  if line.lower().startswith("content-length:")))
                assert 0 <= length <= 5_000_000
                end = split + 4 + length
                if len(buffer) >= end:
                    message = json.loads(buffer[split + 4:end])
                    del buffer[:end]
                    if predicate(message):
                        return message
                    continue
            ready, _, _ = select.select([process.stdout], [], [], max(0, deadline - time.monotonic()))
            if not ready:
                break
            chunk = os.read(process.stdout.fileno(), 65536)
            if not chunk:
                raise AssertionError("LSP closed stdout before the expected reply")
            buffer.extend(chunk)
        raise AssertionError("LSP reply timed out")

    try:
        send("initialize", {"processId": None, "rootUri": None, "capabilities": {}}, 1)
        assert "capabilities" in receive(lambda m: m.get("id") == 1)["result"]
        send("initialized", {})
        uri = "file:///audit.fhirpath"
        send("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "fhirpath",
             "version": 1, "text": "Patient."}})
        diagnostics = receive(lambda m: m.get("method") == "textDocument/publishDiagnostics")
        assert diagnostics["params"]["uri"] == uri
        assert isinstance(diagnostics["params"]["diagnostics"], list)
        send("textDocument/completion", {"textDocument": {"uri": uri},
             "position": {"line": 0, "character": 8}, "context": {"triggerKind": 2, "triggerCharacter": "."}}, 2)
        result = receive(lambda m: m.get("id") == 2)["result"]
        items = result.get("items", []) if isinstance(result, dict) else result
        assert any(item["label"] == "active" for item in items), items
        send("fhirpath/setContext", {"resourceType": "Patient"})
        send("textDocument/didChange", {"textDocument": {"uri": uri, "version": 2},
             "contentChanges": [{"text": "Patient.name."}]})
        changed = receive(lambda m: m.get("method") == "textDocument/publishDiagnostics")
        assert changed["params"]["uri"] == uri
        send("textDocument/completion", {"textDocument": {"uri": uri},
             "position": {"line": 0, "character": 13},
             "context": {"triggerKind": 2, "triggerCharacter": "."}}, 4)
        result = receive(lambda m: m.get("id") == 4)["result"]
        items = result.get("items", []) if isinstance(result, dict) else result
        assert any(item["label"] == "given" for item in items), items
        send("fhirpath/clearContext")
        send("textDocument/didClose", {"textDocument": {"uri": uri}})
        send("shutdown", request_id=3)
        assert receive(lambda m: m.get("id") == 3).get("error") is None
        send("exit")
        process.stdin.close()
        assert process.wait(timeout=10) == 0
        print("stdio initialize, diagnostics, dynamic-schema completion and shutdown passed")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
