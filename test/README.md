
## To test

### Without JWT authentication

```
docker-compose up
```

```
cargo run
```

```
curl -w "\n" -X POST -H "Content-Type: application/graphql" --data "@heroes.gql" http://localhost:4000
```

Or

```
curl -w "\n" -X POST -H "Content-Type: application/json" --data "@heroes.json" http://localhost:4000
```

### With JWT authentication

```
$ SECRET_KEY_BASE=fb9f0a56c2195aa7294f7b076d145bb1a701decd06e8e32cbfdc2f3146a11b3637c5b77d2f98ffb5081af31ae180b69bf2b127ff2496f3c252fcaa20c89d1b019a4639fd26056b6136dd327d118c7d833b357d673d4ba79f1997c4d1d47b74549e0b0e827444fe36dcd7411c0a1384140121e099343d074b6a34c9179ed4687d cargo run
```

```
curl -w "\n" -X POST -H "Content-Type: application/graphql" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo" --data "@heroes.gql" http://localhost:4000
```

Or

```
curl -w "\n" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo" --data "@heroes.json" http://localhost:4000
```

### Testing RBAC

Using a JWT with `"admin"` role:

```
curl -w "\n" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJsb2NhbGhvc3QiLCJzdWIiOiIxIiwicm9sZXMiOiJhZG1pbiJ9.OWRGbi-54ERS5stXrvJaofZL23HVbGEzyGmz-YCXbOE" --data "@admin_only.json" http://localhost:4000
```

## Benchmarking (using `ab`)

```
ab -p test/multi.json -T "application/json" -n 1000 -c 10 --data http://127.0.0.1:4000/
```

With JWT and `"admin"` role:

```
ab -p test/admin_only.json -T "application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJsb2NhbGhvc3QiLCJzdWIiOiIxIiwicm9sZXMiOiJhZG1pbiJ9.OWRGbi-54ERS5stXrvJaofZL23HVbGEzyGmz-YCXbOE" -n 1000 -c 10  http://127.0.0.1:4000/
```

### As a Proxy (with Authentication)

```
curl -w "\n" -x localhost:4000 -X POST -H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo"  --data "@heroes.json" http://localhost:3000/graphql
```

## InfluxDB and Grafana

```
docker exec -it influxdb influx -execute 'create database glances'
```

```
glances --export influxdb
```

```
curl -G 'http://localhost:8086/query?pretty=true' --data-urlencode "db=glances" --data-urlencode "q=SELECT * from \"localhost.cpu\" limit 1"
```
