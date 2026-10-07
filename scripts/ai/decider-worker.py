"""Optional pinned quantized CPU player. Same validated JSONL protocol as Laya."""
import contextlib
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import resource
import sys
import time


def emit(value):
    print(json.dumps(value, allow_nan=False), flush=True)


def main():
    cfg = json.loads(Path(__file__).with_name("decider-config.json").read_text())
    started = time.perf_counter()
    with contextlib.redirect_stdout(sys.stderr):
        from huggingface_hub import snapshot_download
        from decider.infer import Decider
        if importlib.metadata.version("decider-ai") != cfg["sdk"]:
            raise RuntimeError("Decider SDK pin mismatch")
        folder = snapshot_download(cfg["model"], revision=cfg["revision"],
                                   allow_patterns=[cfg["file"], "*.json", "*.jinja"],
                                   local_dir=os.environ.get("BLACKSITE_DECIDER_CACHE", ".blacksite/decider-model"))
        weights = Path(folder) / cfg["file"]
        with weights.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if weights.stat().st_size != cfg["bytes"] or digest != cfg["sha256"]:
            raise RuntimeError("Decider weights checksum mismatch")
        agent = Decider(folder, gguf_file=cfg["file"], gguf_options={
            "n_gpu_layers": 0, "n_threads": cfg["threads"], "n_ctx": cfg["contextTokens"],
        })
    emit({"type": "ready", "device": "cpu", "sdk": cfg["sdk"], "revision": cfg["revision"],
          "version": cfg["version"], "quantization": "Q4_K_M", "weightBytes": cfg["bytes"],
          "loadSeconds": time.perf_counter() - started,
          "peakRssMiB": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
    for line in sys.stdin:
        request = json.loads(line)
        if request.get("type") == "stop":
            break
        started = time.perf_counter()
        try:
            names = list(request["questions"])
            questions = [{"question": request["questions"][n]["instructions"],
                          "options": [f"{k}: {v}" for k, v in request["questions"][n]["criteria"].items()]}
                         for n in names]
            context = json.dumps(request["state"], separators=(",", ":"))
            state_tokens = len(agent.m.tok.encode(context))
            if state_tokens > 1536:
                raise RuntimeError("Observation exceeds the bounded context budget")
            with contextlib.redirect_stdout(sys.stderr):
                result = agent.decide(context, questions,
                                      max_ctx_tokens=cfg["contextTokens"])
            if len(result) != len(names):
                raise RuntimeError("Decider returned wrong question count")
            answers = {}
            for name, row in zip(names, result):
                choices = request["questions"][name]["criteria"]
                matching = [k for k, v in choices.items() if row["choice"] == f"{k}: {v}"]
                if len(matching) != 1:
                    raise RuntimeError("Decider returned invalid choice")
                answers[name] = {"type": "choice", "choice": matching[0]}
            emit({"type": "decision", "id": request["id"], "answers": answers,
                  "usage": {"stateTokens": state_tokens, "truncated": False},
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
