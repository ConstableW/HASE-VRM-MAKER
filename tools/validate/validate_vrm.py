"""Validiert VRMC_vrm einer .vrm-Datei gegen die offiziellen Schemas (vrm-c/vrm-specification, raw.githubusercontent.com).
Aufruf: python validate_vrm.py <datei.vrm> [schema-cache-dir]   (benötigt: jsonschema, referencing)
"""
import json, struct, sys, glob, os, urllib.request
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012

BASE = "https://raw.githubusercontent.com/vrm-c/vrm-specification/master/specification/VRMC_vrm-1.0/schema/"
cache = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(__file__), ".schema-cache")
os.makedirs(cache, exist_ok=True)

def fetch(name):
    p = os.path.join(cache, name)
    if not os.path.exists(p):
        if name in ("glTFProperty.schema.json", "glTFid.schema.json"):
            body = ({"type": "object"} if name.startswith("glTFP") else {"type": "integer", "minimum": 0})
            open(p, "w").write(json.dumps(body))
        else:
            open(p, "wb").write(urllib.request.urlopen(BASE + name, timeout=30).read())
    return json.load(open(p))

todo, seen = ["VRMC_vrm.schema.json"], {}
while todo:
    n = todo.pop()
    if n in seen:
        continue
    seen[n] = fetch(n)
    todo += [x for x in json.dumps(seen[n]).replace('"', " ").split() if x.endswith(".schema.json") and x not in seen]

reg = Registry()
for n, s in seen.items():
    r = Resource.from_contents(s, default_specification=DRAFT202012)
    reg = reg.with_resource(n, r)
    if "$id" in s:
        reg = reg.with_resource(s["$id"], r)

d = open(sys.argv[1], "rb").read()
n = struct.unpack("<I", d[12:16])[0]
ext = json.loads(d[20:20 + n])["extensions"]["VRMC_vrm"]
errs = list(Draft202012Validator(seen["VRMC_vrm.schema.json"], registry=reg).iter_errors(ext))
print(f"{sys.argv[1]}: {len(errs)} Schemafehler")
for e in errs[:10]:
    print(" ", e.message[:160], list(e.path))
sys.exit(1 if errs else 0)
