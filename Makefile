.PHONY: help cross-linux cross-linux-arm cross-test cross-all clean

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

cross-test: ## Run tests for Linux x86_64 using cross
	cross test --target x86_64-unknown-linux-gnu

cross-all: cross-linux cross-linux-arm ## Build for all Linux target platforms
	@echo "All Linux cross builds complete (Windows: use CI/CD)"

clean: ## Clean build artifacts
	cargo clean
