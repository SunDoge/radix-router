# Radix-Router
[![Build Status](https://travis-ci.org/SunDoge/radix-router.svg?branch=master)](https://travis-ci.org/SunDoge/radix-router)

Radix-Router is a Rust port of [julienschmidt/httprouter](https://github.com/julienschmidt/httprouter).

## Examples
An echo server example is written. You can test it by running

```bash
$ cargo run --example echo
```

```bash
$ curl http://127.0.0.1:3000/echo
Try POSTing data to /echo

$ curl -d "param1=SunDoge&param2=TripleZ" -X POST http://127.0.0.1:3000/echo
param1=SunDoge&param2=TripleZ
```
