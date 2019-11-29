## To test

### Without JWT authentication

```
docker-compose up
```

```
cargo run
```

```
curl -w "\n" -X POST -H "Content-Type: application/graphql" --data "@test/heroes.gql" http://localhost:4000
```

Or

```
curl -w "\n" -X POST -H "Content-Type: application/json" --data "@test/heroes.json" http://localhost:4000
```

### With JWT authentication

First, set the `$SECRET_KEY_BASE` environment variable to hold our signing key:

```
$ export SECRET_KEY_BASE=SECRET_KEY_BASE=fb9f0a56c2195aa7294f7b076d145bb1a701decd06e8e32cbfdc2f3146a11b3637c5b77d2f98ffb5081af31ae180b69bf2b127ff2496f3c252fcaa20c89d1b019a4639fd26056b6136dd327d118c7d833b357d673d4ba79f1997c4d1d47b74549e0b0e827444fe36dcd7411c0a1384140121e099343d074b6a34c9179ed4687d
```

Next, run arboric with the test config which uses that environment variable's value as the signing key (decoded from hex):

```
$ cargo run -- -f etc/arboric/config.yml
```

Now, create a valid JWT token using the `jwt` utility ([jwt-cli](https://github.com/mike-engel/jwt-cli)):

```
$ export JWT=$(jwt encode --secret @etc/arboric/secret_key_bytes --iss localhost --sub "1" -P roles=admin)
```

We can now verify that authentication works:

```
$ curl -w "\n" -X POST -H "Content-Type: application/graphql" -H "Authorization: Bearer ${JWT}" --data "@test/heroes.gql" http://localhost:4000
```

Or

```
$ curl -w "\n" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${JWT}" --data "@test/heroes.json" http://localhost:4000
{"data":{"hero":{"id":"3","name":"R2-D2","friends":[{"id":"1","name":"Luke"},{"id":"4","name":"C-3PO"}]}}}
```

Whereas unauthorized requests receive HTTP 401:

```
curl -v -w "\n" -X POST -H "Content-Type: application/json" --data "@test/heroes.json" http://localhost:4000
```

### Testing RBAC

Using a JWT with `"admin"` role:

```
curl -w "\n" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${JWT}" --data "@admin_only.json" http://localhost:4000
```

## Benchmarking (using `ab`)

```
ab -p test/multi.json -T "application/json" -n 1000 -c 10 --data http://127.0.0.1:4000/
```

With JWT and `"admin"` role:

```
ab -p test/admin_only.json -T "application/json" -H "Authorization: Bearer ${JWT}" -n 1000 -c 10 http://127.0.0.1:4000/
```

### As a Proxy (with Authentication)

```

curl -w "\n" -x localhost:4000 -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${JWT}" --data "@heroes.json" http://localhost:3000/graphql

```

## InfluxDB and Grafana

```
docker exec -it influxdb influx -execute 'create database glances'
```

```
glances --export influxdb
```

```
curl -G 'http://localhost:8086/query?pretty=true' --data-urlencode "db=glances" --data-urlencode 'q=SELECT * from "localhost.cpu" limit 1'
```
