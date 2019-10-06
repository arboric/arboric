---
title: "Announcing Arboric GraphQL API Gateway"
date: 2019-09-03T18:00:00+08:00
draft: false
---

In case you haven’t heard, GraphQL is the new Web service API standard that’s rapidly gaining adoption and popularity.

Developers like it because it makes it easier to prototype, develop, consume and maintain APIs whether for React SPA/PWA apps or Flutter mobile apps.

I particularly like how GraphQL frees us from CRUD (or HTTP REST verbs) thinking and aligns extremely well with Domain Driven Design (DDD), Command Query Responsibility Segregation (CQRS) and Event Sourcing, making it easier to design and implement APIs as distributed microservices even for complex domains.

# API Managers

If you’ve been working on large-scale APIs for some time now, you’re sure to have encountered, likely even used an API manager for your REST APIs.

**API Managers** or API gateways provide a variety of features that ease the development and operation of Web services / APIs, such as:

* TLS/SSL(HTTPS) termination
* Routing/forwarding to distributed back-end services
* Caching
* Throttling/DDoS protection
* Authentication
* Authorisation (using ACLs or access control lists)
* Audit or metering
* Content transformation

Some API management services even provide integrated billing services for providers looking to monetise their APIs.

As we were developing our hotel & resort property management SaaS, [Shore Suite](https://www.shoresuite.com/) and other APIs for our consulting engagements, it quickly became apparent though that _all_ existing API gateways or API managers out there are REST-centric and _not_ equipped to handle GraphQL queries.

Since GraphQL only exposes a single endpoint, and generally uses only HTTP POST, traditional query path-based metering and access controls simply won’t work!

It was with this motivation that brought us to develop **Arboric** – the _first_ API manager dedicated for GraphQL.

# Arboric–the Dedicated GraphQL API Manager

The core Arboric API gateway is written in [Rust](https://www.rust-lang.org/) and will be open source and free to use.

We chose Rust because of its promise of stronger guarantees around safety and memory use, while at the same time compiling to blazing-fast native code, _without_ the performance hit of a garbage collector. Rust is hopefully the ‘modern’ C++ alternative we’ve been looking for, that can outperform Java and/or Go.

The initial “proof of concept” version 0.1 is feature complete and demonstrates the following capabilities:

* Authentication and validation of JWT bearer tokens against a secret signing key
* Parsing of GraphQL requests
* Authorisation of requests against a whitelist of roles mapped to queries/fields
* Logging of queries to InfluxDB

It’s available at https://github.com/arboric/arboric

Next milestone is a major refactor and code cleanup, with an eye towards making the core gateway “production ready” (for, um, ‘modest’ meanings of the word _production_).

# Project Goals

**Arboric** is an ambitious project, and we’re looking to implement several things over the coming months, including:

* TLS edge termination
* Mutual TLS to back-end services
* GraphQL schema stitching

We welcome early adopters and contributors!
