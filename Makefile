.PHONY: build install install-config clean

build:
	cargo build --release

install: build
	mkdir -p ~/.config/zellij/plugins
	cp target/wasm32-wasip1/release/zellij_popup.wasm ~/.config/zellij/plugins/zellij-popup.wasm
	@echo ""
	@echo "Plugin installed to ~/.config/zellij/plugins/zellij-popup.wasm"
	@echo ""
	@echo "To install the config and layout files, run:"
	@echo "  make install-config"

install-config:
	mkdir -p ~/.config/zellij/layouts
	cp examples/popup.kdl ~/.config/zellij/layouts/popup.kdl
	@echo ""
	@echo "Layout installed to ~/.config/zellij/layouts/popup.kdl"
	@echo ""
	@echo "Add the keybindings to your ~/.config/zellij/config.kdl by running:"
	@echo "  cat examples/config-snippet.kdl >> ~/.config/zellij/config.kdl"
	@echo ""
	@echo "Or use the complete config:"
	@echo "  cp examples/complete-config.kdl ~/.config/zellij/config.kdl"

clean:
	cargo clean
