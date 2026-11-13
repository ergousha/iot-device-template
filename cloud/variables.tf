variable "debug_telemetry_logs" {
  description = "Route the devices' readings to CloudWatch Logs. Costs per message; enable for a debugging session and switch it off again."
  type        = bool
  default     = false
}
