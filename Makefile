BUILD ?= debug
TARGET=wasm32-wasip2

ifeq ($(BUILD),release)
	WASM_SRC := target/$(TARGET)/release/component_features.wasm
	CARGO_FLAGS := --target $(TARGET) --release
else
	WASM_SRC := target/$(TARGET)/debug/component_features.wasm
	CARGO_FLAGS := --target $(TARGET)
endif

# jco settings

JCO_OUT_DIR = \
			  target/jco

JCO_FLAGS = \
			--base64-cutoff=99999999 \
			--no-nodejs-compat \
			--tla-compat

# ui typescript index.html settings
#
# UIのフロントエンドエントリーポイント

INDEX_HTML = \
		   index.html

ENTRY_TS = \
		   main.ts

# host logic javasript settings
#
# javasriptからrustに提供されるべき機能

WIT_DEPS = \
			component-features/wit/deps/dummy.wit \
			component-features/wit/deps/config_mock.wit \

JS_SHIMS = \
		   js/dummy-logger.js \
		   js/config-mock.js \

JS_MAPS = \
			--map 'dummy:logger/logger=./dummy-logger.js' \
			--map 'config-mock:config-mock/config-mock=./config-mock.js' \


# rust client state logic
#
# RustからjavascriptのUIに提供すべき機能

WIT_EXPORTS = \
			component-features/wit/world.wit

RS_SRCS = \
			component-features/src/lib.rs \
			component-features/src/shift_manager.rs \
			component-features/src/shift_gen.rs \

# ビルド成果物の出力ディレクトリ及びファイル

OUT_DIR = \
		   dist

OUT_HTML = \
		   dist/index.html

$(WASM_SRC): $(RS_SRCS) $(WIT_EXPORTS) $(WIT_DEPS)
	cargo build $(CARGO_FLAGS)

# generate grue code
$(JCO_OUT_DIR): $(WASM_SRC) $(JS_SHIMS)
	npx jco transpile $(WASM_SRC) \
	-o $(JCO_OUT_DIR) \
	$(JCO_FLAGS) \
	$(JS_MAPS)
	# ---
	cp $(JS_SHIMS) $(JCO_OUT_DIR)

gen-jco:$(JCO_OUT_DIR)

$(OUT_HTML): $(INDEX_HTML) gen-jco $(ENTRY_TS)
	# typescriptで型チェックを挟む
	bunx tsc --noEmit

	bun build $(INDEX_HTML) --minify --production --target browser --outdir=$(OUT_DIR)
	bunx vite build

bun-bundle:$(OUT_HTML)

clean: 
	rm -f $(OUT_HTML)
	rm -rf $(JCO_OUT_DIR)
	rm -rf $(OUT_DIR)
	cargo clean

.PHONY: gen-jco bun-bundle mono-html clean

