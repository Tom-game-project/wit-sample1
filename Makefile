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

# guest impls
RS_SRCS = \
		  component-features/src/lib.rs

OUT_DIR = \
		   dist

$(WASM_SRC): $(RS_SRCS)
	cargo build $(CARGO_FLAGS)

# generate grue code
$(JCO_OUT_DIR): $(WASM_SRC) $(JS_SHIMS)
	npx jco transpile $(WASM_SRC) \
	-o $(JCO_OUT_DIR) \
	$(JCO_FLAGS) \
	--map 'dummy:logger/logger=./dummy-logger.js'
	# ---
	cp $(JS_SHIMS) $(JCO_OUT_DIR)

gen-jco:$(JCO_OUT_DIR)

$(JS_TARGET): $(JS_ENTRY) gen-jco
	npx bun build $(JS_ENTRY) --minify --production --target browser --outdir=$(OUT_DIR)

bun-bundle:$(JS_TARGET)

clean: 
	rm $(JS_TARGET)
	rm -rf $(JCO_OUT_DIR)
	rm -rf $(OUT_DIR)
	cargo clean

.PHONY: gen-jco bun-bundle clean

