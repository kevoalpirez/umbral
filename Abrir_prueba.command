#!/bin/bash
cd -- "$(dirname -- "$0")" || exit 1
if ! command -v python3 >/dev/null 2>&1; then
  echo "Necesitas Python 3 para esta prueba. Consulta README.md."
  read -r -p "Pulsa Enter para cerrar."
  exit 1
fi
python3 - <<'APP'
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from functools import partial
import webbrowser
try:
    server=ThreadingHTTPServer(('127.0.0.1',1420),partial(SimpleHTTPRequestHandler,directory='src'))
except OSError:
    print('El puerto 1420 ya está ocupado. Cierra la otra prueba y vuelve a intentarlo.')
    input('Pulsa Enter para cerrar.')
    raise SystemExit(1)
print('Umbral: http://localhost:1420 — cierra con Control+C. No cierres sin guardar.')
webbrowser.open('http://localhost:1420')
try:
    server.serve_forever()
except KeyboardInterrupt:
    server.server_close()
APP
