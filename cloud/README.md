# cloud/

The product's own AWS resources (optional), outside the platform's state.
State: `products/<id>/cloud.tfstate` in the platform's state bucket. The
owner applies it locally; CI only checks it: `terraform validate` and tflint
with the platform's policies (iot-device-sdk `tflint/policies`, at the SDK
revision `Cargo.toml` pins). The rules are in the SDK's `PLATFORM.md` §10;
the short version:

- IoT rules are named `<id>_*`, tagged `killable = "true"`, never `SELECT *`.
  The platform's kill switch disables every enabled `<id>_*` rule when its
  guard trips, and a reset enables exactly those again.
- DynamoDB tables: on-demand, `on_demand_throughput` capped at 1–5 writes a
  second. Lambdas: no reserved concurrency. Log groups: 1–3 days.
- Names `<id>-*` (log groups `/<id>/*`); the provider tags everything
  `Product = <id>`.

The example here is a readings rule, off by default:

```sh
aws-vault exec <owner> -- terraform init
aws-vault exec <owner> -- terraform apply -var debug_telemetry_logs=true   # on
aws-vault exec <owner> -- terraform apply                                  # off again
```

Delete this directory if the product needs no cloud resources.
