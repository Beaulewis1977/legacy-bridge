terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
  }
  
  backend "s3" {
    bucket         = "legacybridge-terraform-state"
    key            = "prod/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "legacybridge-terraform-locks"
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "LegacyBridge"
      Environment = var.environment
      ManagedBy   = "Terraform"
      CostCenter  = var.cost_center
    }
  }
}

# VPC Module
module "vpc" {
  source = "./modules/vpc"
  
  name               = "${var.project_name}-${var.environment}"
  cidr               = var.vpc_cidr
  availability_zones = var.availability_zones
  
  public_subnets  = var.public_subnets
  private_subnets = var.private_subnets
  
  enable_nat_gateway = true
  single_nat_gateway = var.environment != "prod"
  enable_vpn_gateway = var.enable_vpn
  
  tags = {
    "kubernetes.io/cluster/${local.cluster_name}" = "shared"
  }
}

# EKS Cluster
module "eks" {
  source = "./modules/eks"
  
  cluster_name    = local.cluster_name
  cluster_version = var.kubernetes_version
  
  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  control_plane_subnet_ids       = module.vpc.private_subnets
  
  cluster_endpoint_private_access = true
  cluster_endpoint_public_access  = true
  cluster_endpoint_public_access_cidrs = var.allowed_cidr_blocks
  
  enable_irsa = true
  
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
  
  eks_managed_node_groups = {
    general = {
      desired_size = var.node_group_desired_size
      min_size     = var.node_group_min_size
      max_size     = var.node_group_max_size
      
      instance_types = var.node_instance_types
      
      capacity_type = "ON_DEMAND"
      
      labels = {
        Environment = var.environment
        NodeGroup   = "general"
      }
      
      taints = []
      
      update_config = {
        max_unavailable_percentage = 50
      }
    }
    
    spot = {
      desired_size = var.spot_node_group_desired_size
      min_size     = var.spot_node_group_min_size
      max_size     = var.spot_node_group_max_size
      
      instance_types = var.spot_instance_types
      
      capacity_type = "SPOT"
      
      labels = {
        Environment = var.environment
        NodeGroup   = "spot"
        WorkloadType = "batch"
      }
      
      taints = [
        {
          key    = "spot"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      ]
    }
  }
  
  tags = local.common_tags
}

# RDS Database
module "rds" {
  source = "./modules/rds"
  
  identifier     = "${var.project_name}-${var.environment}"
  engine         = "postgres"
  engine_version = var.postgres_version
  instance_class = var.db_instance_class
  
  allocated_storage     = var.db_allocated_storage
  max_allocated_storage = var.db_max_allocated_storage
  storage_encrypted     = true
  
  database_name = "legacybridge"
  username      = "legacybridge"
  
  vpc_id                  = module.vpc.vpc_id
  subnet_ids              = module.vpc.private_subnets
  allowed_security_groups = [module.eks.node_security_group_id]
  
  backup_retention_period = var.environment == "prod" ? 30 : 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  skip_final_snapshot = var.environment != "prod"
  deletion_protection = var.environment == "prod"
  
  performance_insights_enabled = var.environment == "prod"
  monitoring_interval         = var.environment == "prod" ? 60 : 0
  
  tags = local.common_tags
}

# S3 Buckets
module "s3" {
  source = "./modules/s3"
  
  bucket_prefix = "${var.project_name}-${var.environment}"
  
  create_artifacts_bucket = true
  create_backups_bucket   = true
  create_logs_bucket      = true
  
  enable_versioning = var.environment == "prod"
  enable_encryption = true
  
  lifecycle_rules = {
    artifacts = {
      enabled = true
      transition_days = 30
      expiration_days = 90
    }
    logs = {
      enabled = true
      transition_days = 7
      expiration_days = 30
    }
  }
  
  tags = local.common_tags
}

# IAM Roles for Service Accounts (IRSA)
resource "aws_iam_role" "legacybridge_app" {
  name = "${local.cluster_name}-legacybridge-app"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRoleWithWebIdentity"
      Effect = "Allow"
      Principal = {
        Federated = module.eks.oidc_provider_arn
      }
      Condition = {
        StringEquals = {
          "${module.eks.oidc_provider}:sub" = "system:serviceaccount:default:legacybridge"
          "${module.eks.oidc_provider}:aud" = "sts.amazonaws.com"
        }
      }
    }]
  })
}

# Application IAM Policies
resource "aws_iam_role_policy" "legacybridge_s3" {
  name = "${local.cluster_name}-legacybridge-s3"
  role = aws_iam_role.legacybridge_app.id
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject"
        ]
        Resource = [
          "${module.s3.artifacts_bucket_arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "s3:ListBucket"
        ]
        Resource = [
          module.s3.artifacts_bucket_arn
        ]
      }
    ]
  })
}

# Configure Kubernetes Provider
provider "kubernetes" {
  host                   = module.eks.cluster_endpoint
  cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
  
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    command     = "aws"
    args        = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
  }
}

# Configure Helm Provider
provider "helm" {
  kubernetes {
    host                   = module.eks.cluster_endpoint
    cluster_ca_certificate = base64decode(module.eks.cluster_certificate_authority_data)
    
    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      command     = "aws"
      args        = ["eks", "get-token", "--cluster-name", module.eks.cluster_name]
    }
  }
}

# Deploy LegacyBridge Helm Chart
resource "helm_release" "legacybridge" {
  name       = "legacybridge"
  chart      = "../helm/legacybridge"
  namespace  = "default"
  
  values = [
    templatefile("${path.module}/helm-values.yaml", {
      environment        = var.environment
      image_tag         = var.app_version
      database_host     = module.rds.endpoint
      database_name     = module.rds.database_name
      database_username = module.rds.username
      ingress_hostname  = var.app_hostname
      aws_region        = var.aws_region
      iam_role_arn      = aws_iam_role.legacybridge_app.arn
    })
  ]
  
  depends_on = [
    module.eks,
    module.rds
  ]
}

# Outputs
output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "database_endpoint" {
  description = "RDS database endpoint"
  value       = module.rds.endpoint
  sensitive   = true
}

output "app_url" {
  description = "Application URL"
  value       = "https://${var.app_hostname}"
}

locals {
  cluster_name = "${var.project_name}-${var.environment}"
  common_tags = {
    Project     = var.project_name
    Environment = var.environment
    ManagedBy   = "Terraform"
    CostCenter  = var.cost_center
  }
}