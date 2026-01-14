use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "t-aws")]
#[command(about = "AWS CLI installer & quick reference tool")]
#[command(version)]
pub struct Cli {
    /// Install AWS CLI directly (skip TUI)
    #[arg(short, long)]
    pub install: bool,

    /// Uninstall AWS CLI directly (skip TUI)
    #[arg(short, long)]
    pub uninstall: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show S3 command cheatsheet
    S3,
    /// Show EC2 command cheatsheet  
    Ec2,
    /// Show IAM command cheatsheet
    Iam,
    /// List AWS resources in region
    Resources {
        /// AWS region (e.g., ap-northeast-2)
        #[arg(short, long)]
        region: Option<String>,
    },
}

pub fn print_s3_cheatsheet() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════════╗
║                    📦 AWS S3 Quick Reference                     ║
╠══════════════════════════════════════════════════════════════════╣
║  BUCKET OPERATIONS                                               ║
╠══════════════════════════════════════════════════════════════════╣
║  aws s3 ls                        # List all buckets             ║
║  aws s3 mb s3://bucket-name       # Create bucket                ║
║  aws s3 rb s3://bucket-name       # Delete bucket (empty)        ║
║  aws s3 rb s3://bucket-name --force  # Delete bucket (with files)║
╠══════════════════════════════════════════════════════════════════╣
║  FILE OPERATIONS                                                 ║
╠══════════════════════════════════════════════════════════════════╣
║  aws s3 ls s3://bucket-name       # List files in bucket         ║
║  aws s3 ls s3://bucket/prefix/    # List files with prefix       ║
║  aws s3 cp file.txt s3://bucket/  # Upload file                  ║
║  aws s3 cp s3://bucket/file.txt . # Download file                ║
║  aws s3 mv s3://bucket/a.txt s3://bucket/b.txt  # Rename/move    ║
║  aws s3 rm s3://bucket/file.txt   # Delete file                  ║
╠══════════════════════════════════════════════════════════════════╣
║  SYNC & BATCH                                                    ║
╠══════════════════════════════════════════════════════════════════╣
║  aws s3 sync ./local s3://bucket  # Sync local → S3              ║
║  aws s3 sync s3://bucket ./local  # Sync S3 → local              ║
║  aws s3 cp . s3://bucket --recursive  # Upload folder            ║
║  aws s3 rm s3://bucket --recursive    # Delete all files         ║
╠══════════════════════════════════════════════════════════════════╣
║  USEFUL OPTIONS                                                  ║
╠══════════════════════════════════════════════════════════════════╣
║  --recursive          # Apply to all files in folder             ║
║  --exclude "*.log"    # Exclude pattern                          ║
║  --include "*.txt"    # Include pattern                          ║
║  --dryrun             # Preview without executing                ║
║  --acl public-read    # Set public access                        ║
╚══════════════════════════════════════════════════════════════════╝
"#
    );
}

pub fn print_ec2_cheatsheet() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════════╗
║                    🖥️  AWS EC2 Quick Reference                   ║
╠══════════════════════════════════════════════════════════════════╣
║  INSTANCE MANAGEMENT                                             ║
╠══════════════════════════════════════════════════════════════════╣
║  aws ec2 describe-instances       # List all instances           ║
║  aws ec2 start-instances --instance-ids i-xxx  # Start           ║
║  aws ec2 stop-instances --instance-ids i-xxx   # Stop            ║
║  aws ec2 reboot-instances --instance-ids i-xxx # Reboot          ║
║  aws ec2 terminate-instances --instance-ids i-xxx  # Terminate   ║
╠══════════════════════════════════════════════════════════════════╣
║  INSTANCE INFO                                                   ║
╠══════════════════════════════════════════════════════════════════╣
║  aws ec2 describe-instances --query \                            ║
║    'Reservations[].Instances[].[InstanceId,State.Name,Tags]'     ║
║  aws ec2 describe-instance-status --instance-ids i-xxx           ║
╠══════════════════════════════════════════════════════════════════╣
║  SECURITY GROUPS                                                 ║
╠══════════════════════════════════════════════════════════════════╣
║  aws ec2 describe-security-groups # List security groups         ║
║  aws ec2 authorize-security-group-ingress \                      ║
║    --group-id sg-xxx --protocol tcp --port 22 --cidr 0.0.0.0/0   ║
╠══════════════════════════════════════════════════════════════════╣
║  KEY PAIRS                                                       ║
╠══════════════════════════════════════════════════════════════════╣
║  aws ec2 describe-key-pairs       # List key pairs               ║
║  aws ec2 create-key-pair --key-name MyKey  # Create new key      ║
╚══════════════════════════════════════════════════════════════════╝
"#
    );
}

pub fn print_iam_cheatsheet() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════════╗
║                    🔐 AWS IAM Quick Reference                    ║
╠══════════════════════════════════════════════════════════════════╣
║  USER MANAGEMENT                                                 ║
╠══════════════════════════════════════════════════════════════════╣
║  aws iam list-users               # List all users               ║
║  aws iam create-user --user-name xxx  # Create user              ║
║  aws iam delete-user --user-name xxx  # Delete user              ║
║  aws iam get-user --user-name xxx     # Get user info            ║
╠══════════════════════════════════════════════════════════════════╣
║  ACCESS KEYS                                                     ║
╠══════════════════════════════════════════════════════════════════╣
║  aws iam list-access-keys --user-name xxx  # List keys           ║
║  aws iam create-access-key --user-name xxx # Create key          ║
║  aws iam delete-access-key --access-key-id xxx --user-name xxx   ║
╠══════════════════════════════════════════════════════════════════╣
║  ROLES & POLICIES                                                ║
╠══════════════════════════════════════════════════════════════════╣
║  aws iam list-roles               # List roles                   ║
║  aws iam list-policies            # List policies                ║
║  aws iam attach-user-policy --user-name xxx \                    ║
║    --policy-arn arn:aws:iam::aws:policy/ReadOnlyAccess           ║
╠══════════════════════════════════════════════════════════════════╣
║  CONFIGURE                                                       ║
╠══════════════════════════════════════════════════════════════════╣
║  aws configure                    # Set up credentials           ║
║  aws configure list               # Show current config          ║
║  aws sts get-caller-identity      # Who am I?                    ║
╚══════════════════════════════════════════════════════════════════╝
"#
    );
}
