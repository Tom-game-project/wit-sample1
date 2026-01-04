BUILD ?= debug
TARGET=wasm32-wasip2

ifeq ($(BUILD),release)
	WASM_SRC := target/$(TARGET)/release/component_features.wasm
	CARGO_FLAGS := --target $(TARGET) --release
else
	WASM_SRC := target/$(TARGET)/debug/component_features.wasm
	CARGO_FLAGS := --target $(TARGET)
endif

JCO_OUT_DIR = \
			  target/jco

JCO_FLAGS = \
			--base64-cutoff=99999999 \
			--no-nodejs-compat \
			--tla-compat

JS_ENTRY = \
		   index.html

JS_TARGET = \
			build.js

# host impls
JS_SHIMS = \
		   js/dummy-logger.js \
		   js/config-mock.js \

# guest impls
RS_SRCS = \
		  component-features/src/lib.rs

OUT_DIR = \
		   dist

OUT_HTML = dist/index.html

$(WASM_SRC): $(RS_SRCS)
	cargo build $(CARGO_FLAGS)

# generate grue code
$(JCO_OUT_DIR): $(WASM_SRC) $(JS_SHIMS)
	npx jco transpile $(WASM_SRC) \
	-o $(JCO_OUT_DIR) \
	$(JCO_FLAGS) \
	--map 'dummy:logger/logger=./dummy-logger.js' \
	--map 'config-mock:config-mock/config-mock=./config-mock.js'
	# ---
	cp $(JS_SHIMS) $(JCO_OUT_DIR)

gen-jco:$(JCO_OUT_DIR)

$(OUT_HTML): $(JS_ENTRY) gen-jco
	npx bun build $(JS_ENTRY) --minify --production --target browser --outdir=$(OUT_DIR)

bun-bundle:$(OUT_HTML)

mono-html: bun-bundle $(OUT_HTML)
	npx vite build

clean: 
	rm $(OUT_HTML)
	rm -rf $(JCO_OUT_DIR)
	rm -rf $(OUT_DIR)
	cargo clean

.PHONY: gen-jco bun-bundle mono-html clean

