# The product's own cloud resources (platform plan §7, PLATFORM.md §10),
# outside the platform's state. The owner applies them locally:
#
#   aws-vault exec <owner> -- terraform init
#   aws-vault exec <owner> -- terraform apply [-var debug_telemetry_logs=true]
#
# The example: the devices' readings (src/app.rs) reach the rule
# <product>_readings over Basic Ingest ($aws/rules/<product>_readings/<thing>,
# no messaging fee), which writes them to a 3-day log group. Off by default.
# The thing is clientid(), which the platform pins to the thing name; never a
# payload field or topic(), which the device controls (PLATFORM.md §6).
# Remove or replace it with what the product needs, within the cost rules
# the CI checks (tflint with the platform's policies).
#
# Telemetry also needs the platform's switch: the devices publish only while
# their `platform` shadow has desired.telemetry.enabled = true.

data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

# Confused-deputy guard: IoT may assume the role only for this account's readings rule. The
# ARN is built as a string: the rule resource references the role.
data "aws_iam_policy_document" "rule_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["iot.amazonaws.com"]
    }
    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [data.aws_caller_identity.current.account_id]
    }
    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = ["arn:aws:iot:${data.aws_region.current.region}:${data.aws_caller_identity.current.account_id}:rule/${local.product}_readings"]
    }
  }
}

resource "aws_cloudwatch_log_group" "readings" {
  name              = "/${local.product}/readings"
  retention_in_days = 3
}

resource "aws_iam_role" "readings_rule" {
  name               = "${local.product}-readings-rule"
  assume_role_policy = data.aws_iam_policy_document.rule_assume.json
}

resource "aws_iam_role_policy" "readings_rule" {
  name = "${local.product}-readings-rule"
  role = aws_iam_role.readings_rule.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["logs:CreateLogStream", "logs:PutLogEvents"]
      Resource = "${aws_cloudwatch_log_group.readings.arn}:*"
    }]
  })
}

resource "aws_iot_topic_rule" "readings" {
  name        = "${local.product}_readings"
  description = "Device readings (Basic Ingest) -> CloudWatch Logs. Off unless debug_telemetry_logs."
  enabled     = var.debug_telemetry_logs
  sql         = "SELECT clientid() AS thing, fw, up_s, heap, timestamp() AS ts FROM 'dt/+/readings'"
  sql_version = "2016-03-23"
  tags        = { killable = "true" }

  cloudwatch_logs {
    log_group_name = aws_cloudwatch_log_group.readings.name
    role_arn       = aws_iam_role.readings_rule.arn
  }
}
