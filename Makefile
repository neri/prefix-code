.PHONY: all clean test server
.SUFFIXED: .wasm .js .rs .ts

TS_ROOT	= ts/
TS_SRC	= $(TS_ROOT)src/
TS_LIB	= $(TS_ROOT)lib/
TS_DIST	= $(TS_ROOT)dist/
TS_MAIN	= $(TS_DIST)main.js
RS_SRC	= rs/
RS_LIB	= $(TS_ROOT)lib/libentropy.wasm

all: $(RS_LIB) $(TS_MAIN)

clean:
	(cd $(RS_SRC); cargo clean)
	-rm $(RS_LIB) $(TS_MAIN)
	-rm -rf $(TS_DIST)

update:
	cargo update
	(cd $(TS_ROOT); pnpm update)

debug:
	(cd $(RS_SRC); cargo build)
	cp target/wasm32-unknown-unknown/debug/libimage.wasm $(RS_LIB)

$(RS_LIB): $(RS_SRC)src/*.rs
	(cd $(RS_SRC); wasm-pack build --target web --out-dir ../$(TS_LIB))

$(TS_MAIN): $(RS_LIB) $(TS_SRC)*.ts
	echo "export const HASH = \"`git rev-parse --short HEAD`\";" > $(TS_ROOT)src/hash.ts
	(cd $(TS_ROOT); pnpm i; pnpm run build)

server:
	(cd $(TS_ROOT); pnpm dev)
