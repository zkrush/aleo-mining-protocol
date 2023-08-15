### Aleo-mining-protocol V0.1

![image-20230815151444019](https://cdn.jsdelivr.net/gh/ghostant-1017/img@main/img/image-20230815151444019.png)

1.`Clien` send `AuthRequest` to `Pool`

```json
{
    "type": "auth_request",
    "username": "test_username"
}
```

`username` should be a miner account belong to `Pool` or an aleo `Address`

2.`Pool` send `AuthResponse` to `Client`

```json
{
    "type": "auth_response",
    "result": true,
    "address": "aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px",
    "message": null
}
```

3.`Pool` send `Task` to `Client`

```json
{
    "type": "task",
    "epoch_challenge": {
        "epoch_number": 0,
        "epoch_block_hash": "ab1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5g436j",
        "degree": 8191
    },
    "difficulty": 1
}
```

4.`Client` send `Solution` to `Pool`

```json
{
    "type": "solution",
    "solution": {
        "partial_solution": {
            "address": "aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px",
            "nonce": 17434404712108178625,
            "commitment": "puzzle17y0u6jxd4ywgp65aekm4t5j9yc5rfm3n8y49vm36lftpkecdt7fsdtx9z2kgde4mm4vac270zelgqezpl00"
        },
        "proof.w": {
            "x": "5313204516602391617578260248738022071572239071617674251205960352549628692580589949066636165162788432007586213586",
            "y": "71036653811392212446198086635018125758811736522567984443760491619503963418585905446717771586361166967319050026503",
            "infinity": false
        }
    },
    "epoch_number": 0
}
```

