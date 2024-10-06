# VARIABLES
BINARY_NAME := matchmaker
HTTP_MAIN_PACKAGE_PATH := src/http
MIGRATION_FOLDER := migrations
DB_URL := postgres://bulutgocer:pass@localhost:5432/matchmaker?sslmode=disable

# DOCUMENTATION
## Database Documentation
.PHONY: db-docs
db-docs:
	dbdocs build doc/db.dbml

# HELPERS
## Help and Confirmation
.PHONY: help
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

.PHONY: confirm
confirm:
	@echo -n 'Are you sure? [y/N] ' && read ans && [ $${ans:-N} = y ]

## Git State Check
.PHONY: no-dirty
no-dirty:
	git diff --exit-code

# QUALITY CONTROL
## Code Formatting and Checks
.PHONY: format
format:
	cargo fmt

.PHONY: tidy
tidy:
	cargo fmt
	cargo check

.PHONY: audit
audit:
	cargo check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo audit

## Clean-up
.PHONY: clean
clean:
	cargo clean

# DEVELOPMENT AND TESTING
## Testing
.PHONY: test
test:
	cargo test --verbose

.PHONY: test-release
test-release:
	cargo test --release --verbose

.PHONY: test-specific
test-specific:
	@echo "Usage: make test-specific TEST=<test_name>"
	cargo test --verbose $(TEST)

.PHONY: test-doc
test-doc:
	cargo test --doc --verbose

.PHONY: test-lib
test-lib:
	cargo test --lib --verbose

.PHONY: test-bin
test-bin:
	cargo test --bin $(BINARY_NAME) --verbose

.PHONY: test-examples
test-examples:
	cargo test --examples --verbose

.PHONY: test-bench
test-bench:
	cargo bench

.PHONY: test-features
test-features:
	cargo test --all-features --verbose

.PHONY: test-no-default-features
test-no-default-features:
	cargo test --no-default-features --verbose

# BUILD AND RUN
## Development and Execution
.PHONY: watch
watch:
	cargo watch -q -c -w src/ -x run

# DOCKER
## Docker Compose Operations
.PHONY: docker-compose
docker-compose: docker-compose-stop docker-compose-start

.PHONY: docker-compose-start
docker-compose-start:
	docker-compose up --build

.PHONY: docker-compose-stop
docker-compose-stop:
	docker-compose down

## Core Docker Dependencies
.PHONY: docker-dependency-start
docker-dependency-start:
	docker-compose -f docker-compose-core.yaml up -d

# SQLx Operations
.PHONY: sqlx-install
sqlx-install:
	cargo install sqlx-cli

.PHONY: sqlx-db-create
sqlx-db-create:
	sqlx database create --database-url $(DB_URL)

.PHONY: sqlx-db-drop
sqlx-db-drop:
	sqlx database drop --database-url $(DB_URL)

.PHONY: sqlx-migrate-add
sqlx-migrate-add:
	@echo "Usage: make sqlx-migrate-add NAME=<name>"
	sqlx migrate add -r $(NAME) --source $(MIGRATION_FOLDER)

.PHONY: sqlx-migrate-run
sqlx-migrate-run:
	sqlx migrate run --source $(MIGRATION_FOLDER) --database-url $(DB_URL)

.PHONY: sqlx-migrate-revert
sqlx-migrate-revert:
	sqlx migrate revert --source $(MIGRATION_FOLDER) --database-url $(DB_URL)

.PHONY: sqlx-prepare
sqlx-prepare:
	cargo sqlx prepare --database-url $(DB_URL)
