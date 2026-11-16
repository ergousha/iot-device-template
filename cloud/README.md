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
- DynamoDB tables: on-demand, `on_demand_throughput` capped at 1–5 reads
  and 1–5 writes a second. Lambdas: no reserved concurrency. Log groups:
  1–3 days.
- Names `<id>-*` (log groups `/<id>/*`); the provider tags everything
  `Product = <id>`.
- No `aws_iot_role_alias` or `aws_iot_authorizer` (device identity is the
  platform's), and IoT rules never republish (an MQTT loop).
- No unauthenticated ingress: a Lambda function URL is `AWS_IAM`, an API
  Gateway method or route has an authorization (not `NONE`).
- Every `aws_s3_bucket.X` has `aws_s3_bucket_public_access_block.X` with all
  four blocks `true`.

The example here is a readings rule, off by default:

```sh
aws-vault exec <owner> -- terraform init
aws-vault exec <owner> -- terraform apply -var debug_telemetry_logs=true   # on
aws-vault exec <owner> -- terraform apply                                  # off again
```

Delete this directory if the product needs no cloud resources.
