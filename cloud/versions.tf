terraform {
  required_version = ">= 1.10.0" # use_lockfile needs 1.10

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }

  # The platform's state bucket, this product's key: SSM
  # /iot-platform/products/<id>/platform names both, and `ergo product init`
  # writes them here. Backend blocks take no variables.
  backend "s3" {
    bucket       = "esp32-ztp-tfstate-637423178579"
    key          = "products/template/cloud.tfstate"
    region       = "eu-central-1"
    encrypt      = true
    use_lockfile = true
  }
}

locals {
  # The product id; `ergo product init` sets it. Rules are named
  # "${local.product}_*" so the platform's kill switch can disable them.
  product = "template"
}

provider "aws" {
  region = "eu-central-1"

  default_tags {
    tags = {
      Product   = local.product
      ManagedBy = "terraform"
    }
  }
}
