IMAGE ?= ai-manager-dev:latest
CONTAINERDIR ?= /workspace
CURDIR := $(shell pwd)

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "%-20s %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the development image
	docker build -t $(IMAGE) .

.PHONY: run
run: ## Run bash in the container (mounts repo)
	docker run --rm -it \
		--user "$$(id -u):$$(id -g)" \
		-v "$(CURDIR)":"$(CONTAINERDIR)" \
		-w "$(CONTAINERDIR)" \
		$(IMAGE) bash

.PHONY: dev
dev: ## Run Vite dev server in the container (port 1420)
	docker run --rm -it \
		-p 1420:1420 \
		-e TAURI_DEV_HOST=0.0.0.0 \
		--user "$$(id -u):$$(id -g)" \
		-v "$(CURDIR)":"$(CONTAINERDIR)" \
		-w "$(CONTAINERDIR)" \
		$(IMAGE) npm run dev

.PHONY: tauri-dev
tauri-dev: ## Run Tauri dev in the container (requires host X11/Wayland display)
	docker run --rm -it \
		-e DISPLAY=$${DISPLAY:-:0} \
		-v /tmp/.X11-unix:/tmp/.X11-unix \
		-p 1420:1420 \
		-e TAURI_DEV_HOST=0.0.0.0 \
		--user "$$(id -u):$$(id -g)" \
		-v "$(CURDIR)":"$(CONTAINERDIR)" \
		-w "$(CONTAINERDIR)" \
		$(IMAGE) npm run tauri dev

.PHONY: lockfile
lockfile: ## Regenerate package-lock.json inside Linux container (keeps libc metadata)
	docker run --rm \
		--user "$$(id -u):$$(id -g)" \
		-v "$(CURDIR)":"$(CONTAINERDIR)" \
		-w "$(CONTAINERDIR)" \
		$(IMAGE) npm install

.PHONY: clean
clean: ## Remove the local Docker image
	docker rmi $(IMAGE)
