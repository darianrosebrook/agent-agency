# Agent Agency V3 - AWS Terraform Variables

variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod"
  }
}

variable "project_name" {
  description = "Project name"
  type        = string
  default     = "agent-agency"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "eks_cluster_version" {
  description = "EKS cluster version"
  type        = string
  default     = "1.28"
}

variable "node_groups" {
  description = "EKS node groups configuration"
  type = map(object({
    instance_types = list(string)
    min_size       = number
    max_size       = number
    desired_size   = number
    labels         = map(string)
    taints = optional(list(object({
      key    = string
      value  = string
      effect = string
    })), [])
  }))
  default = {
    general = {
      instance_types = ["t3.large"]
      min_size       = 3
      max_size       = 10
      desired_size   = 3
      labels = {
        Environment = "prod"
        NodeGroup   = "general"
      }
    }
    compute = {
      instance_types = ["c6i.xlarge"]
      min_size       = 1
      max_size       = 5
      desired_size   = 2
      labels = {
        Environment = "prod"
        NodeGroup   = "compute"
        Workload    = "ai-inference"
      }
      taints = [
        {
          key    = "workload"
          value  = "ai-inference"
          effect = "NO_SCHEDULE"
        }
      ]
    }
  }
}

variable "database_config" {
  description = "Database configuration"
  type = object({
    instance_class       = string
    allocated_storage    = number
    max_allocated_storage = number
    backup_retention     = number
    multi_az             = bool
  })
  default = {
    instance_class        = "db.r6g.large"
    allocated_storage     = 100
    max_allocated_storage = 1000
    backup_retention      = 30
    multi_az              = true
  }
}

variable "redis_config" {
  description = "Redis configuration"
  type = object({
    node_type      = string
    num_cache_nodes = number
  })
  default = {
    node_type       = "cache.t4g.medium"
    num_cache_nodes = 2
  }
}

variable "opensearch_config" {
  description = "OpenSearch configuration"
  type = object({
    instance_type = string
    instance_count = number
    volume_size    = number
  })
  default = {
    instance_type = "t3.medium.elasticsearch"
    instance_count = 2
    volume_size    = 20
  }
}

variable "monitoring_config" {
  description = "Monitoring configuration"
  type = object({
    enable_prometheus = bool
    enable_grafana    = bool
    enable_jaeger     = bool
    retention_days    = number
  })
  default = {
    enable_prometheus = true
    enable_grafana    = true
    enable_jaeger     = true
    retention_days    = 30
  }
}

variable "security_config" {
  description = "Security configuration"
  type = object({
    enable_waf           = bool
    enable_cloudtrail    = bool
    enable_config        = bool
    enable_guardduty     = bool
    enable_security_hub  = bool
  })
  default = {
    enable_waf          = true
    enable_cloudtrail   = true
    enable_config       = true
    enable_guardduty    = true
    enable_security_hub = true
  }
}

variable "backup_config" {
  description = "Backup configuration"
  type = object({
    enable_rds_backup     = bool
    enable_s3_backup      = bool
    backup_retention_days = number
    cross_region_backup   = bool
  })
  default = {
    enable_rds_backup     = true
    enable_s3_backup      = true
    backup_retention_days = 30
    cross_region_backup   = false
  }
}

variable "tags" {
  description = "Common tags for all resources"
  type        = map(string)
  default = {
    Project     = "Agent Agency"
    ManagedBy   = "Terraform"
    Version     = "3.0.0"
  }
}

# Secrets (should be provided via environment or secret management)
variable "database_password" {
  description = "Database password"
  type        = string
  sensitive   = true
  default     = "CHANGE_ME_IN_PRODUCTION"
}

variable "redis_password" {
  description = "Redis password"
  type        = string
  sensitive   = true
  default     = ""
}

variable "openai_api_key" {
  description = "OpenAI API key"
  type        = string
  sensitive   = true
  default     = ""
}

variable "anthropic_api_key" {
  description = "Anthropic API key"
  type        = string
  sensitive   = true
  default     = ""
}

variable "jwt_secret" {
  description = "JWT signing secret"
  type        = string
  sensitive   = true
  default     = "CHANGE_ME_IN_PRODUCTION"
}

variable "encryption_key" {
  description = "Data encryption key"
  type        = string
  sensitive   = true
  default     = "CHANGE_ME_IN_PRODUCTION"
}
