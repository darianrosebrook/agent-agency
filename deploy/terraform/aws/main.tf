# Agent Agency V3 - AWS EKS Deployment
# Terraform configuration for complete AWS infrastructure

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
  }

  backend "s3" {
    bucket = "agent-agency-terraform-state"
    key    = "eks/terraform.tfstate"
    region = "us-east-1"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "Agent Agency"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Version     = "3.0.0"
    }
  }
}

# VPC Configuration
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "agent-agency-vpc"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  enable_nat_gateway     = true
  single_nat_gateway     = false
  enable_dns_hostnames   = true
  enable_dns_support     = true

  # Database subnets
  database_subnets    = ["10.0.201.0/24", "10.0.202.0/24", "10.0.203.0/24"]
  create_database_subnet_group = true

  # Tags for EKS
  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = "1"
  }

  public_subnet_tags = {
    "kubernetes.io/role/elb" = "1"
  }
}

# EKS Cluster
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  cluster_name    = "agent-agency-${var.environment}"
  cluster_version = "1.28"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  # EKS Managed Node Groups
  eks_managed_node_groups = {
    general = {
      instance_types = ["t3.large"]
      min_size       = 3
      max_size       = 10
      desired_size   = 3

      labels = {
        Environment = var.environment
        NodeGroup   = "general"
      }

      tags = {
        "k8s.io/cluster-autoscaler/enabled" = "true"
      }
    }

    compute = {
      instance_types = ["c6i.xlarge"]
      min_size       = 1
      max_size       = 5
      desired_size   = 2

      labels = {
        Environment = var.environment
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

      tags = {
        "k8s.io/cluster-autoscaler/enabled" = "true"
      }
    }

    # GPU nodes (uncomment when needed)
    # gpu = {
    #   instance_types = ["g4dn.xlarge"]
    #   min_size       = 0
    #   max_size       = 3
    #   desired_size   = 0
    #
    #   labels = {
    #     Environment = var.environment
    #     NodeGroup   = "gpu"
    #     Accelerator = "nvidia"
    #   }
    #
    #   taints = [
    #     {
    #       key    = "nvidia.com/gpu"
    #       value  = "present"
    #       effect = "NO_SCHEDULE"
    #     }
    #   ]
    #
    #   tags = {
    #     "k8s.io/cluster-autoscaler/enabled" = "true"
    #   }
    # }
  }

  # Cluster access
  manage_aws_auth_configmap = true

  aws_auth_roles = [
    {
      rolearn  = module.cluster_autoscaler_irsa_role.iam_role_arn
      username = "cluster-autoscaler"
      groups   = ["system:masters"]
    }
  ]

  aws_auth_users = [
    {
      userarn  = "arn:aws:iam::123456789012:user/admin"
      username = "admin"
      groups   = ["system:masters"]
    }
  ]

  # Addons
  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }
}

# IRSA for Cluster Autoscaler
module "cluster_autoscaler_irsa_role" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name                        = "agent-agency-cluster-autoscaler"
  attach_cluster_autoscaler_policy = true

  oidc_providers = {
    ex = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:cluster-autoscaler"]
    }
  }
}

# RDS PostgreSQL Database
module "db" {
  source  = "terraform-aws-modules/rds/aws"
  version = "~> 6.0"

  identifier = "agent-agency-${var.environment}"

  engine               = "postgres"
  engine_version       = "15.4"
  family               = "postgres15"
  major_engine_version = "15"
  instance_class       = "db.r6g.large"

  allocated_storage     = 100
  max_allocated_storage = 1000

  db_name  = "agent_agency"
  username = "agent_agency"
  port     = 5432

  multi_az               = true
  db_subnet_group_name   = module.vpc.database_subnet_group
  vpc_security_group_ids = [module.security_group_db.security_group_id]

  maintenance_window              = "Mon:00:00-Mon:03:00"
  backup_window                   = "03:00-06:00"
  enabled_cloudwatch_logs_exports = ["postgresql", "upgrade"]
  create_cloudwatch_log_group     = true

  backup_retention_period = 30
  skip_final_snapshot     = false
  final_snapshot_identifier = "agent-agency-${var.environment}-final"

  performance_insights_enabled          = true
  performance_insights_retention_period = 7
  create_monitoring_role                = true
  monitoring_interval                   = 60

  parameters = [
    {
      name  = "autovacuum"
      value = "1"
    },
    {
      name  = "client_encoding"
      value = "utf8"
    }
  ]
}

# ElastiCache Redis
module "redis" {
  source  = "terraform-aws-modules/elasticache/aws"
  version = "~> 1.0"

  replication_group_id = "agent-agency-${var.environment}"
  description          = "Redis cluster for Agent Agency"

  engine_version             = "7.0"
  port                       = 6379
  node_type                  = "cache.t4g.medium"
  num_cache_clusters         = 2
  parameter_group_name       = "default.redis7"
  preferred_cache_cluster_azs = ["${var.aws_region}a", "${var.aws_region}b"]

  maintenance_window = "mon:03:00-mon:04:00"
  apply_immediately  = true

  # Security
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                 = random_password.redis_auth_token.result

  subnet_ids = module.vpc.database_subnets
  security_group_ids = [module.security_group_redis.security_group_id]
}

# OpenSearch (Elasticsearch)
module "opensearch" {
  source  = "terraform-aws-modules/opensearch/aws"
  version = "~> 1.0"

  domain_name = "agent-agency-${var.environment}"
  engine_version = "OpenSearch_2.7"

  cluster_config = {
    instance_type          = "t3.medium.elasticsearch"
    instance_count         = 2
    zone_awareness_enabled = true
    availability_zone_count = 2
  }

  ebs_options = {
    ebs_enabled = true
    volume_size = 20
    volume_type = "gp3"
  }

  encrypt_at_rest = {
    enabled = true
  }

  domain_endpoint_options = {
    enforce_https       = true
    tls_security_policy = "Policy-Min-TLS-1-2-2019-07"
  }

  node_to_node_encryption = {
    enabled = true
  }

  vpc_options = {
    subnet_ids         = slice(module.vpc.private_subnets, 0, 2)
    security_group_ids = [module.security_group_opensearch.security_group_id]
  }

  access_policies = data.aws_iam_policy_document.opensearch_access_policy.json

  log_publishing_options = [
    {
      log_type                 = "INDEX_SLOW_LOGS"
      cloudwatch_log_group_arn = aws_cloudwatch_log_group.opensearch_slow_logs.arn
    },
    {
      log_type                 = "SEARCH_SLOW_LOGS"
      cloudwatch_log_group_arn = aws_cloudwatch_log_group.opensearch_slow_logs.arn
    }
  ]
}

# S3 Bucket for artifacts and backups
module "s3_artifacts" {
  source  = "terraform-aws-modules/s3-bucket/aws"
  version = "~> 3.0"

  bucket = "agent-agency-${var.environment}-artifacts-${random_string.suffix.result}"

  versioning = {
    enabled = true
  }

  server_side_encryption_configuration = {
    rule = {
      apply_server_side_encryption_by_default = {
        sse_algorithm = "AES256"
      }
    }
  }

  lifecycle_configuration_rules = [
    {
      enabled = true
      id      = "artifacts_lifecycle"
      expiration = {
        days = 365
      }
      filter = {
        prefix = "artifacts/"
      }
    },
    {
      enabled = true
      id      = "backups_lifecycle"
      transition = {
        days          = 30
        storage_class = "STANDARD_IA"
      }
      expiration = {
        days = 365
      }
      filter = {
        prefix = "backups/"
      }
    }
  ]
}

# CloudWatch Log Groups
resource "aws_cloudwatch_log_group" "opensearch_slow_logs" {
  name              = "/aws/opensearch/agent-agency-${var.environment}"
  retention_in_days = 30
}

# Security Groups
module "security_group_db" {
  source  = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "agent-agency-db"
  description = "Security group for database"
  vpc_id      = module.vpc.vpc_id

  ingress_with_cidr_blocks = [
    {
      from_port   = 5432
      to_port     = 5432
      protocol    = "tcp"
      description = "PostgreSQL access from within VPC"
      cidr_blocks = module.vpc.vpc_cidr_block
    },
  ]
}

module "security_group_redis" {
  source  = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "agent-agency-redis"
  description = "Security group for Redis"
  vpc_id      = module.vpc.vpc_id

  ingress_with_cidr_blocks = [
    {
      from_port   = 6379
      to_port     = 6379
      protocol    = "tcp"
      description = "Redis access from within VPC"
      cidr_blocks = module.vpc.vpc_cidr_block
    },
  ]
}

module "security_group_opensearch" {
  source  = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "agent-agency-opensearch"
  description = "Security group for OpenSearch"
  vpc_id      = module.vpc.vpc_id

  ingress_with_cidr_blocks = [
    {
      from_port   = 443
      to_port     = 443
      protocol    = "tcp"
      description = "HTTPS access from within VPC"
      cidr_blocks = module.vpc.vpc_cidr_block
    },
  ]
}

# IAM Policies and Roles
resource "aws_iam_policy" "agent_agency_secrets" {
  name        = "AgentAgencySecretsAccess"
  description = "Allow access to secrets for Agent Agency"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue",
          "secretsmanager:DescribeSecret"
        ]
        Resource = [
          "arn:aws:secretsmanager:${var.aws_region}:*:secret:agent-agency/*"
        ]
      }
    ]
  })
}

# Random values for unique resource names
resource "random_string" "suffix" {
  length  = 8
  special = false
  upper   = false
}

resource "random_password" "redis_auth_token" {
  length  = 32
  special = false
}

# Data sources
data "aws_iam_policy_document" "opensearch_access_policy" {
  statement {
    effect = "Allow"

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    actions = [
      "es:*"
    ]

    resources = [
      "arn:aws:es:${var.aws_region}:*:domain/agent-agency-${var.environment}/*"
    ]

    condition {
      test     = "IpAddress"
      variable = "aws:SourceIp"
      values   = [module.vpc.vpc_cidr_block]
    }
  }
}

# Outputs
output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "database_endpoint" {
  description = "RDS database endpoint"
  value       = module.db.db_instance_address
}

output "redis_endpoint" {
  description = "Redis cluster endpoint"
  value       = module.redis.elasticache_replication_group_primary_endpoint_address
}

output "opensearch_endpoint" {
  description = "OpenSearch domain endpoint"
  value       = module.opensearch.domain_endpoint
}

output "artifacts_bucket" {
  description = "S3 bucket for artifacts"
  value       = module.s3_artifacts.s3_bucket_id
}
