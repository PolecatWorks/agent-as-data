.PHONY: all build-fe build-be compose-db-up compose-db-down compose-db-clean robot-test

DATABASE_URL ?= postgres://postgres:mysecretpassword@localhost:5432/aaddb

all: build-be build-fe

compose-db-up:
	docker compose -f docker-compose/postgres.yaml up -d

compose-db-down:
	docker compose -f docker-compose/postgres.yaml down

compose-db-clean:
	docker compose -f docker-compose/postgres.yaml down -v

build-be:
	docker build -t agent-as-data-be:latest aad-be-container

build-fe:
	docker build -t agent-as-data-fe:latest aad-fe-container

robot-test:
	./integration-tests/run-tests-local.sh
