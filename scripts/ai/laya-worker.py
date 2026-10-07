"""Test-only persistent CPU inference worker. Stdout is JSONL RPC, never logging."""
import contextlib
import json
import os
from pathlib import Path
import resource
import sys
import time

os.environ.setdefault("USE_TF", "0")
os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")


def emit(value):
    print(json.dumps(value, allow_nan=False), flush=True)


def main():
    config = json.loads(Path(__file__).with_name("config.json").read_text())
    started = time.perf_counter()
    # Some dependency versions print diagnostics: keep the protocol unambiguous.
    with contextlib.redirect_stdout(sys.stderr):
        import laya
        import torch
        torch.set_num_threads(config["threads"])
        torch.set_num_interop_threads(1)
        torch.manual_seed(config["modelSeed"])
        torch.use_deterministic_algorithms(True)
        if laya.__version__ != config["sdk"]:
            raise RuntimeError(f"Expected Laya {config['sdk']}, got {laya.__version__}")
        agent = laya.load(config["model"], subfolder=config["subfolder"],
                          revision=config["revision"], device="cpu", backend="eager")
    emit({"type": "ready", "device": str(agent.device), "sdk": laya.__version__,
          "revision": agent.revision, "parameters": sum(p.numel() for p in agent.model.parameters()),
          "loadSeconds": time.perf_counter() - started,
          "peakRssMiB": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
    for line in sys.stdin:
        request = json.loads(line)
        if request.get("type") == "stop":
            break
        started = time.perf_counter()
        try:
            with contextlib.redirect_stdout(sys.stderr):
                result = agent.predict(request["state"], request["questions"], max_len=1024)
            emit({"type": "decision", "id": request["id"], "answers": result["answers"], "usage": result.get("usage", {}),
                  "seconds": time.perf_counter() - started,
                  "peakRssMiB": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
        except Exception as error:
            emit({"type": "error", "id": request["id"], "message": str(error)})


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        emit({"type": "error", "message": str(error)})
        sys.exit(1)
