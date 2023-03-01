## How to replicate

1. Authenticate with firehose endpoint
2. `cd safe`
3. `cargo build --target wasm32-unknown-unknown --release`
4. `substreams run -e polygon.streamingfast.io:443 substreams.yaml map_safe_events --start-block 39146200 --stop-block 39146298`
   Should trigger parallel processing mode

## Error Message

```
Error: rpc error: code = Internal desc = error building pipeline: failed setup request: parallel processing run: scheduler run: process job result for target "deployment:factory:store_factories": worker ended in error: receiving stream resp: rpc error: code = ResourceExhausted desc = grpc: received message larger than max (6021390 vs. 4194304)
```
