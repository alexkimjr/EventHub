# EventHub API

Small Actix Web service that accepts JSON POSTs at /ingest and forwards them to Kafka.

Configuration: `config.yaml` with `server` and `kafka` tables.

Logging: uses `tracing` and Actix Logger middleware.

Steps for my current configuration:

1. Start Kafka consumer consumer:
```
kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic first_topics --from-beginning
```
2. Build and run: `cargo run`.
3. POST using postman 


## CI
--
added GitHub Actions workflow in `.github/workflows/ci.yml` that runs unit tests and an end-to-end test which uses `docker-compose` to bring up Kafka, Zookeeper and the API.

## Docker
------
Build and run everything (Zookeeper, Kafka, API):
1. Build and start with docker-compose:
	docker-compose up --build
2. API will be available at http://localhost:8080; config is mounted from `config.yaml`.
To run only the API container (expects Kafka reachable at the addresses in `config.yaml`):
	docker build -t eventhub_api:local .
	docker run --rm -p 8080:8080 -v ${PWD}/config.yaml:/app/config.yaml eventhub_api:local