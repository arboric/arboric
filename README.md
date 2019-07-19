Arboric GraphQL API Gateway
====

## Proof-of-Concept Roadmap

* [x] JWT Authentication
* [ ] Logging to InfluxDB
* [ ] Role and Path-based Access Control Lists (black/white list)

## To test

### Without JWT authentication

```
docker run --rm -p 3000:80 aisrael/graphql-dotnet-examples
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

```
$ SECRET_KEY_BASE=fb9f0a56c2195aa7294f7b076d145bb1a701decd06e8e32cbfdc2f3146a11b3637c5b77d2f98ffb5081af31ae180b69bf2b127ff2496f3c252fcaa20c89d1b019a4639fd26056b6136dd327d118c7d833b357d673d4ba79f1997c4d1d47b74549e0b0e827444fe36dcd7411c0a1384140121e099343d074b6a34c9179ed4687d cargo run
```

-H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo"

```
curl -w "\n" -X POST -H "Content-Type: application/graphql" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo" --data "@test/heroes.gql" http://localhost:4000
```

Or

```
curl -w "\n" -X POST -H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo" --data "@test/heroes.json" http://localhost:4000
```

### As a Proxy (with Authentication)

```
curl -w "\n" -x localhost:4000 -X POST -H "Content-Type: application/json" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE1NjAyMTk4MjUsImlzcyI6ImRlbW8uc2hvcmVzdWl0ZS5kZXYiLCJzdWIiOiIxNyJ9.AGHOUJKQ7cOX_buVVbbsIarYfU_C_pwOeoAlhVkNceo"  --data "@test/heroes.json" http://localhost:3000/graphql
```
