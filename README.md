# Carol - A programmable third party for any protocol

Carol is a [*serverless*](https://en.wikipedia.org/wiki/Serverless_computing) computing service that runs user uploaded programs.
At the moment the programs run on only one machine but in the end goal is to have the programs be run between *federations* of carol nodes.
The purpose of Carols is not to "outsource" computing to the cloud like in traditional compute providers.
A user asks Carol to run their program so that others can have confidence that the program will be run faithfully without having to trust a single party.


## Roadmap

### 1. Stateless oracles

The first milesone is to have carol functioning as a programmable BLS-based DLC oracle for the protocol described in *[Cryptographic Oracle-Based Conditional Payments]*.
To do this it doesn't need state and it doesn't need to communicate with other carol nodes.

### 2. TODO Local state

At some point we want to allow programs to store state that they can access when they are re-activated as well as generate secret keys for `secp256k1`, `bls12_381`



### 3. TODO Federated state

Finally, we want programs to be able to store state on multiple carol nodes working as a federation. Secret keys would be Shamir secret shared across these nodes as well.


## Run it

### Generate a config and run

First clone the repository and then `cd` into it and run:

``` sh
cargo run -p carol -- --cfg carol.yml config-gen
```

This will generate a default configuration (along with some secret keys!) and put them in `carol.yml`.

``` sh
cargo run -p carol -- --cfg carol.yml run &
```


### Uploading a program

First compile it to wasm. Here we just compile one of the examples in `example-crates`.

``` sh
wasm_output=$( cargo run -p carlo -- build -p bitmex_guest )
```

Then upload it to carol:

``` sh
carol_url=http://localhost:8000
binary_id=$( cargo run -p carlo -- upload --carol-url "${carol_url}" --binary "${wasm_output}" )
```

Note this `id` is just a hash of the binary. In general it's intended for client software to
generate it locally and check it against what's returned rather than just blindly trusted the carol
server.

### Create the machine

Carol machines are created from a binary and a parameterization array. Most machines will have an empty parameterization for now so we make an empty POST request to t

``` sh
machine_id=$( cargo run -p carlo -- create --carol-url "${carol_url}" --binary-id "${binary_id}" )
```

Note this `id` is a hash of the binary and the (empty) parameterization vector. In general it's
intended for client software to generate it locally and check it against what's returned rather than
just blindly trusted the carol server.

### Activate the machine

The machine we've created has a few ways of activating it as defined [the code](./example-crates/bitmex/guest/src/lib.rs ) we compiled to the binary.
At the time of writing it's:

- `attest_to_price_at_minute`
- `bit_decompose_attest_to_price_at_minute`

Let's send a HTTP request to the machine which will activate the `attest_to_price_at_minute` which will fetch the price of the symbol we ask for at the time we ask for.


```sh
curl -v -XPOST --data-binary '{"time" : "2023-04-16T12:30:00Z", "symbol" : ".BXBT"}' "${carol_url}/machines/${machine_id}/activate/attest_to_price_at_minute"
```

which at the time of writing returns:

``` json
{
  "Ok": {
    "price": 30264,
    "signature": "8d737860c57c0463ab532127359c0a7fbc9fa1bf56b120ad3b724637fb3a3c08d621ce5afe20de25889d14c7e23a0a4a19961cc08596f2c82fd84b9b00fa24b5fc4e67226300d855f6e51176d7ef73525e37d7baad6dae701271a0ede593000d"
  }
}
```


[Cryptographic Oracle-Based Conditional Payments]: https://eprint.iacr.org/2022/499
