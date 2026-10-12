#!/usr/bin/env sh
# Compila EGRESSO in WebAssembly e produce un'unica pagina autonoma:
#   web/dist/egresso.html  (HTML + JS + wasm incorporato in base64)
#
# Requisiti: rustup target add wasm32-unknown-unknown
set -eu
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown --lib
WASM=target/wasm32-unknown-unknown/release/egresso.wasm

mkdir -p web/dist
base64 < "$WASM" | tr -d '\n' > web/dist/egresso.wasm.b64
# Sostituisce il segnaposto nel modello con il modulo codificato
# (letto da file: è troppo lungo per passarlo come argomento).
awk 'BEGIN { getline b64 < "web/dist/egresso.wasm.b64" } { gsub(/__WASM_BASE64__/, b64); print }' web/index.html > web/dist/egresso.html

echo "ok: web/dist/egresso.html ($(wc -c < web/dist/egresso.html) byte, wasm $(wc -c < "$WASM") byte)"
