.PHONY: help cross-linux cross-linux-arm cross-windows cross-test cross-all clean

help: ## Show this help message
	@echo "Cross-compilation make targets for keyring-cli"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^\([a-zA-Z_-]*:\).*##\(.*\)/\1\t\2/p' $(MAKEFILE_LIST) | column -t -s '	'

cross-linux: ## Build for Linux x86_64 using cross
	cross build --target x86_64-unknown-linux-gnu --release

cross-linux-arm: ## Build for Linux ARM64 using cross
	cross build --target aarch64-unknown-linux-gnu --release

cross-windows: ## Build for Windows x86_64 using cross
	cross build --target x86_64-pc-windows-msvc --release

cross-test: ## Run tests for Linux x86_64 using cross
	cross test --target x86_64-unknown-linux-gnu

cross-all: cross-linux cross-linux-arm cross-windows ## Build for all target platforms
	@echo "All cross builds complete"

clean: ## Clean build artifacts
	cargo clean
