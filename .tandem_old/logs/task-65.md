---
id: task-65
type: task
title: "Configure custom domain and GitHub Pages launch workflow"
priority: "low"
parentId: "task-59"
references: ["decision-1"]
relatedFiles: ["site/astro.config.mjs", "site/README.md", ".github/workflows/docs.yml", "docs/guides/docs-site.md", "site/public/CNAME"]
tags: ["docs", "deployment", "github-pages", "dns"]
createdAt: "2026-06-29T20:49:41Z"
updatedAt: "2026-07-04T17:48:53Z"
accord:
  status: "accepted"
  assignee: "herd:task-65"
  claimedAt: "2026-07-04T17:22:36Z"
  deliveredAt: "2026-07-04T17:39:39Z"
  deliverables: ["Production docs URL selected: https://trytandem.dev/", "Astro site/base updated for custom-domain root: site: 'https://trytandem.dev', base: '/'", "Added site/public/CNAME containing trytandem.dev", "Updated site/README.md with GitHub Pages custom-domain setup, gh checks, DNS records, and launch verification commands", "Updated docs/guides/docs-site.md with the same custom-domain launch workflow documentation", "Configured GitHub Pages custom domain via gh api; Pages now reports cname: trytandem.dev", "Documented Namecheap DNS instructions: remove conflicting parking/URL redirect records for apex and www; add apex A records 185.199.108.153, 185.199.109.153, 185.199.110.153, 185.199.111.153; add apex AAAA records 2606:50c0:8000::153, 2606:50c0:8001::153, 2606:50c0:8002::153, 2606:50c0:8003::153 if supported; add www CNAME to Algorant.github.io; use Automatic or 30 min TTL; after propagation retry gh api --method PUT repos/Algorant/tandem/pages -F https_enforced=true and curl probes"]
  validation:
    commands: ["cd site && bun run build passed; 12 pages built", "site/dist/CNAME contains trytandem.dev", "Generated sitemap/canonical URLs use https://trytandem.dev/", "git diff --check passed", "gh workflow list/view verified docs workflow is active", "gh api repos/Algorant/tandem/actions/permissions verified Actions enabled", "gh api repos/Algorant/tandem/pages verified Pages build_type is workflow and current settings are cname: trytandem.dev, https_enforced: false, https_certificate: null"]
  summary: "Accepted after user validation of Namecheap DNS setup and task-65 deliverables. User confirmed task-65 can be considered validated."
  evidence: ["Current DNS is still Namecheap parking/forwarding: trytandem.dev A -> 162.255.119.229; www.trytandem.dev CNAME -> parkingpage.namecheap.com", "HTTPS enforcement attempt currently fails with GitHub error: The certificate does not exist yet", "Full launch verification is blocked on external Namecheap DNS update and GitHub certificate provisioning", "Branch/worktree/commit status: shared main working tree, no separate branch/worktree, no commit created"]
  filesChanged: ["site/astro.config.mjs", "site/public/CNAME", "site/README.md", "docs/guides/docs-site.md"]
  reviewer: "parent/user"
  updatedAt: "2026-07-04T17:48:44Z"
completedAt: "2026-07-04T17:48:53Z"
completion:
  summary: "Configured the docs site for trytandem.dev custom-domain launch. Updated Astro site/base settings, added CNAME artifact source, documented GitHub Pages and Namecheap DNS setup, and used GitHub CLI to configure/inspect Pages custom-domain state."
  validation: "User completed Namecheap DNS setup and validated task-65. Parent verified public DNS propagation on 1.1.1.1 for apex A/AAAA and www CNAME, GitHub Pages custom domain set to trytandem.dev, and worker validation passed docs build and diff check. HTTPS certificate/enforcement may still require propagation time."
  reviewer: "parent/user"
---

## Description

Choose the production docs URL, update Astro site/base settings, add site/public/CNAME if needed, document DNS records, verify GitHub Pages Actions settings, enforce HTTPS, and confirm the deployment URL works for both project Pages and the custom domain path.
