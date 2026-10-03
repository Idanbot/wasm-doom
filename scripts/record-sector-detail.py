#!/usr/bin/env python3
"""Copy a native imagegen deliverable and retain its prompt and source hash."""
from pathlib import Path
import hashlib
import json
import shutil
import sys
import fcntl

root = Path(__file__).resolve().parents[1]
folder = root / 'art/sector-detail'
job = next(j for j in json.loads((folder / 'jobs.json').read_text()) if j['id'] == sys.argv[1])
destination = root / job['source']
shutil.copyfile(sys.argv[2], destination)
record = dict(job, provider='native imagegen', sha256=hashlib.sha256(destination.read_bytes()).hexdigest())
manifest = folder / 'generation.json'
with (folder / '.generation.lock').open('w') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    rows = json.loads(manifest.read_text()) if manifest.exists() else []
    rows = [r for r in rows if r['id'] != record['id']] + [record]
    manifest.write_text(json.dumps(rows, indent=2) + '\n')
print(record['id'], record['sha256'][:8])
