# The product's own cloud resources (platform plan §7, PLATFORM.md §10),
# outside the platform's state. The owner applies them locally:
#
#   aws-vault exec <owner> -- terraform init
#   aws-vault exec <owner> -- terraform apply [-var debug_telemetry_logs=true]
#
# The example: the devices' readings (src/app.rs) reach the rule
# <product>_readings over Basic Ingest ($aws/rules/<product>_readings/<thing>,
# no messaging fee), which writes them to a 3-day log group. Off by default.
# Remove or replace it with what the product needs, within the cost rules
# the CI checks (tflint with the platform's policies).
#
# Telemetry also needs the platform's switch: the devices publish only while
# their `platform` shadow has desired.telemetry.enabled = true.

data "aws_iam_policy_document" "rule_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["iot.amazonaws.com"]
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
  sql         = "SELECT thing, fw, up_s, heap, timestamp() AS ts FROM 'dt/+/readings'"
  sql_version = "2016-03-23"
  tags        = { killable = "true" }

  cloudwatch_logs {
    log_group_name = aws_cloudwatch_log_group.readings.name
    role_arn       = aws_iam_role.readings_rule.arn
  }
}
