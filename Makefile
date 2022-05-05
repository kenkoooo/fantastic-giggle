up:
	docker-compose up

down:
	docker-compose down

test:
	docker-compose exec backend cargo test --workspace -- --test-threads=1

psql:
	docker-compose exec postgres psql postgres://user:password@postgres:5432/test
