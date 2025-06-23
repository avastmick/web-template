# Container hosting platforms for your Rust web application

Your containerized Rust/Svelte application requires hosting that balances cost, performance, and flexibility. After analyzing 40+ platforms across major cloud providers, developer-friendly PaaS solutions, and VPS providers, three distinct deployment strategies emerge that meet your $50/month production budget while supporting SQLite-to-PostgreSQL migration paths and customer infrastructure deployment needs.

## Platform landscape reveals three optimal paths

The container hosting market in 2025 offers dramatically different approaches for your use case. **Google Cloud Run leads serverless options at just $0.40/month for production**, leveraging generous free tiers that cover 180,000 vCPU-seconds and 2 million requests monthly. For teams preferring predictable costs, **AWS Lightsail Containers provides fixed pricing at $7/month** for production environments. The best overall value comes from **Hetzner Cloud at €4.15/month** (approximately $4.50), offering 2 vCPUs, 4GB RAM, and 20TB bandwidth – specifications that typically cost 3-4x more on other platforms.

Each approach serves different operational philosophies. Serverless platforms excel at handling variable traffic patterns but struggle with persistent storage requirements for SQLite. Traditional VPS providers offer complete control and excellent price-performance ratios but require more operational expertise. Developer-friendly PaaS solutions split the difference, providing managed infrastructure with container flexibility.

### Major cloud providers offer sophistication at higher costs

AWS container services present multiple options, though most exceed budget constraints. **App Runner charges $0.064/vCPU-hour for active containers**, resulting in approximately $25-30/month for minimal configurations. ECS Fargate's pay-per-second model works well for batch workloads but becomes expensive for always-on services at roughly $18/month for continuous operation. The sweet spot is **Lightsail Containers at $7/month**, providing 0.25 vCPU and 512MB RAM with 500GB data transfer – sufficient for Rust's efficient resource usage.

Google Cloud Run's pricing model particularly suits Rust applications. The platform's ability to scale to zero combined with Rust's fast startup times creates an ideal match. **Free tier coverage eliminates costs for test environments** and keeps production expenses minimal. However, Cloud Run's lack of proper persistent volume support creates challenges for SQLite deployments, requiring creative solutions like Cloud Storage FUSE mounts that introduce latency concerns.

Azure Container Apps matches Google's pricing structure but offers less mature tooling. **Container Instances provide per-second billing** but become costly for continuous workloads at $33/month for basic configurations. Azure's strength lies in enterprise integration rather than budget optimization.

### Developer PaaS platforms balance ease and flexibility

**Fly.io emerges as the Rust community's preferred platform**, offering global edge deployment across 34+ regions with usage-based pricing. A production setup with two shared-CPU instances and 5GB volume storage costs approximately $17/month. The platform's native support for SQLite volumes and straightforward PostgreSQL migration path addresses your database requirements directly. Fly.io's CLI tooling and documentation specifically mention Rust optimizations, making deployment particularly smooth.

Railway provides an alternative with simpler pricing at $5/month base plus usage. The platform excels at developer experience with GitHub integration and real-time collaboration features. **Production workloads typically cost $20-40/month** depending on traffic patterns. Railway's managed PostgreSQL offerings simplify database migration but lack volume mounting for SQLite.

Render's pricing starts at $7/month for production services with persistent disk support at $0.25/GB/month. While more expensive than alternatives, Render provides comprehensive managed services including auto-scaling and built-in CI/CD. The platform's main limitation is its free tier's 15-minute inactivity timeout, making it unsuitable for production use.

### VPS providers deliver exceptional value for self-managed deployments

**Hetzner Cloud dominates price-performance benchmarks**, offering 8GB RAM and 4 vCPUs for €15.12/month (approximately $16). This configuration handles 100K daily requests with significant headroom while including 20TB bandwidth – a specification that costs $50-100/month elsewhere. The platform's German engineering focus on reliability and GDPR compliance adds value for European deployments.

DigitalOcean provides multiple deployment options. **App Platform starts at $5/month** for basic containers, scaling to $29/month for dedicated resources. Alternatively, Droplets offer traditional VPS hosting from $4/month with one-click Docker installation. The ecosystem includes managed PostgreSQL at $15/month and Kubernetes clusters with free control planes.

Vultr's regular performance tier at $2.50/month represents the absolute minimum viable option for testing environments. Production deployments on $12/month instances (2GB RAM, 55GB SSD) provide good performance with 3TB bandwidth. **Vultr's free Kubernetes control plane** enables container orchestration without additional costs.

## Technical implementation addresses specific requirements

Your scratch-based Rust container presents unique optimization opportunities across platforms. Multi-stage builds reduce final image sizes to 5-10MB, minimizing registry storage costs and deployment times. Static linking with musl eliminates runtime dependencies, improving security and portability.

### SQLite persistence requires platform-specific approaches

**Network-attached storage universally fails for SQLite** due to file locking limitations and latency issues. Successful strategies include:

1. Local container storage with periodic backups to object storage
2. In-memory SQLite with persistent snapshots
3. Early migration to managed PostgreSQL services

Fly.io's volume implementation provides the most SQLite-friendly solution, offering local NVMe storage with snapshot capabilities. Hetzner and other VPS providers enable direct disk mounting, eliminating network storage concerns entirely.

### PostgreSQL migration paths vary by platform

Managed PostgreSQL services cluster around similar price points:
- **Google Cloud SQL**: $9.37/month (db-f1-micro)
- **AWS RDS**: $12.60/month (db.t3.micro)
- **DigitalOcean/Linode**: $15/month (1GB single node)

Self-managed PostgreSQL on VPS instances reduces costs to zero but requires operational expertise. Container-based PostgreSQL deployments using official images provide middle ground with version control and easy backup strategies.

### Traffic handling exceeds requirements on all platforms

Your 100K daily requests translate to 1.16 requests/second average with potential 10-20x peaks. **Every researched platform handles this load comfortably** with minimal resource allocation. Rust's efficient memory usage and fast response times mean even 512MB instances suffice for many workloads.

Auto-scaling capabilities vary significantly. Serverless platforms like Cloud Run scale automatically based on concurrent requests. Kubernetes-based solutions use Horizontal Pod Autoscalers. Traditional VPS deployments require manual scaling or custom automation.

## Security and compliance features vary significantly

Container security scanning integrates seamlessly across platforms using **Trivy for vulnerability detection** and secret scanning. GitHub Actions and GitLab CI/CD pipelines support automated security gates that fail builds on critical vulnerabilities.

Platform-specific security features include:
- **Google Cloud Run**: Automatic binary authorization and VPC Service Controls
- **AWS**: IAM roles for service accounts and VPC security groups
- **Fly.io**: WireGuard-based private networking between services
- **Hetzner**: GDPR compliance and EU data residency

## Customer deployment capabilities require hybrid approaches

**No mainstream PaaS platform supports direct customer infrastructure deployment**, representing a significant limitation. Successful strategies combine:

1. **Docker Compose generation** for customer self-hosting
2. **Helm charts** for Kubernetes deployments
3. **Terraform modules** for cloud provisioning

Northflank's Bring Your Own Cloud (BYOC) feature provides the closest match to requirements but exceeds budget constraints. **The recommended approach uses Fly.io or Hetzner for your deployments** while providing Docker Compose and Kubernetes manifests for customer installations.

## Cost optimization strategies maximize value

Production deployment recommendations by budget tier:

**Ultra-Budget ($15-20/month)**:
- Hetzner Cloud CPX11 (€4.15/month)
- Self-managed PostgreSQL
- Coolify for deployment automation
- Cloudflare for CDN/DDoS protection

**Balanced ($25-35/month)**:
- Fly.io with usage-based scaling
- Fly Postgres for managed database
- Global edge deployment included
- Integrated monitoring and logs

**Managed ($40-50/month)**:
- DigitalOcean App Platform ($29/month)
- Managed PostgreSQL ($15/month)
- Automated SSL and domains
- Built-in CI/CD pipeline

## Testing and deployment automation ensures reliability

Security testing using **Trivy in CI/CD pipelines** catches vulnerabilities before production. Load testing with **k6 validates 100K request/day capacity** while identifying bottlenecks. Chaos engineering tools like **LitmusChaos** verify resilience under failure conditions.

GitHub Actions provides the most cost-effective CI/CD solution with generous free tiers. A complete pipeline includes:
1. Rust compilation with sccache for faster builds
2. Security scanning with cargo-audit and Trivy
3. Multi-architecture Docker builds using buildx
4. Automated deployment to staging and production

## Conclusion

**For immediate deployment, Fly.io offers the best combination** of Rust community support, developer experience, and global performance at $15-17/month. Teams comfortable with infrastructure management should choose **Hetzner Cloud at €15/month** for exceptional value. Organizations prioritizing managed services benefit from **Google Cloud Run's generous free tier** despite SQLite limitations.

The Rust ecosystem's efficiency enables successful deployments on minimal infrastructure. Your scratch-based container approach combined with platform-specific optimizations ensures cost-effective scaling well beyond initial requirements. Customer deployment needs require additional tooling investment but remain achievable through Docker Compose and Kubernetes manifest generation.
