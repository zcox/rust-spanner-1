we're going to build a simple example web service in rust backed by spanner.
axum for http framework
gcloud-spanner for rust/spanner integration
use cloud-spanner-emulator in a docker-compose.yml
in a .env set SPANNER_EMULATOR_HOST for the rust client
the web service should have two simple endpoints:
- a POST that takes a simple json request body and writes it to spanner at some key/ID
- a GET that reads a specified key/ID and returns a json response body
fully document how to run the db and service, and use the endpoints
use curl/jq to verify that everything works end-to-end