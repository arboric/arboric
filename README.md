Arboric GraphQL API Gateway
====

**Current version:** 0.2 Alpha

Arboric is the first, and so far only API proxy / gateway dedicated specifically for GraphQL. It aims to provide several key features:

#### Auditing / Metering

Currently supports logging of query and mutation fields and counts to InfluxDB. In the near future, Arboric aims to:

* allow selectively logging requests metadata such as JWT claims & values
* support logging to Kafka

#### Authentication

Currently, Arboric can enforce verification of a JWT `Authorization: Bearer` token using a supplied HS256 signing key (via environment variable). In the near future, it aims to support:

* Supplying the HS256 signing key as a hex, base64, or 'raw' value from the environment, directly as run-time argument or configuration value, or from a file
* Support for RS256 asymmetric token verification

##### Authorization (ABAC)

Arboric provides Attribute Based Access Control that allows great flexibility in access controls. Currently, it supports matching:

* JWT claim presence
* JWT claim equality
* JWT claim inclusion (e.g. `claims["roles"] includes "admin"` will match `"roles": "user, admin"`)

It also supports `Allow` or `Deny` rules based on GraphQL pattern matching. For example:

* `foo` or `query:foo` matches a query for the field `foo`
* `mutation:doSomething` matches the mutation `doSomething`
* `*` or `query:*` matches any query, while
* `mutation:*` matches any mutation

In the future, Arboric aims to allow:

* nested fields matching, e.g.
  * `hero.secretIdentity`
  * `hero.friends.secretIdentity`
  * `**.secretIdentity`
* matching by GraphQL type, e.g. `type:Hero`
* matching by type _and_ field, e.g. `type:Hero{secretIdentity}`

### Feature Wishlist

* TLS/SSL edge termination
* Two-way TLS certificate authentication/validation from edge to backend
* Multiple listeners on a single server process

## To Use

Currently, Arboric is not yet distributed in binary. In the future, we intend to make Arboric available as binary packages for Linux and Mac OS, as well as a Docker image.

To build Arboric requires:

* Rust 1.37.0


#### Clone this repository

```
git clone https://gitlab.com/arboric/arboric
```

#### Build the binary

```
cargo build --release
```

#### Run `arboric` with the sample config file

```
./target/release/arboric -f etc/arboric/config.yml start
```

That will start Arboric listening on port 4000, forwarding requests to `http://localhost:3001/graphql`, and validating the JWT `Authorization: Bearer` token using the `SECRET_KEY_BASE` (hexadecimal) environment variable value

## Roadmap

### 0.2.1 Alpha 1

* [ ] Configurable log level
* [ ] File logger
* [ ] Configurable policies
* [ ] Configurable InfluxDB logging

### 0.3 Beta

* [ ] Allow for multiple Listeners
* [ ] Arboric API (in GraphQL, of course)
* [ ] Allow for run-time configuration (via the API)

## Versions

### 0.2 Alpha (2019-09-12)

* ABAC (Attribute Based Access Control), allows for
  * Matching by claim presence, equality, or inclusion
* Configuration model
* Read configuration from YAML
* CLI arguments processor

### 0.1 Proof-of-Concept (2019-09-03)

* JWT Authentication
* Logging to InfluxDB
* Role and Path-based Access Control Lists (black/white list)
