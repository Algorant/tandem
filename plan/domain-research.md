# Domain research: `tandem.md`

Date: 2026-07-01
Task: task-67

## Bottom line

`tandem.md` is a valid `.md` country-code domain, but it is already registered and therefore cannot be obtained through normal registration right now. The practical path is either owner acquisition/broker outreach or choosing an available alternative such as `gettandem.md`.

Recommendation: do **not** spend implementation time assuming `tandem.md` is obtainable. If a domain is needed soon, register `gettandem.md` (or another available brand modifier) and set up redirects/canonical docs there. In parallel, optionally monitor `tandem.md` or pursue a broker/acquisition inquiry if the exact name is important.

## Findings

### `.md` registration status

- `.md` is the Moldova ccTLD. IANA delegates WHOIS to `whois.nic.md` and identifies the registry operator as IP Serviciul Tehnologia Informatiei si Securitate Cibernetica (STISC).
- NIC.MD says anyone can register a `.md` domain, first-come/first-served, with domain names of 2-63 letters/numbers/hyphens and no leading/trailing hyphen.
- NIC.MD lists the annual fee as 450 Moldovan lei, shown on the site as about `$25.47` at the time checked.
- Registrar pricing varies widely. Examples found:
  - NIC.MD direct: 450 MDL/year (~$25.47 displayed).
  - TLD-List search result: `.md` registration prices from about `$48.99` to `$373.11` across registrars.
  - Gandi: registration `$288.43`/year; renewal `$551.98`/year; requires Gandi Corporate Services.
  - EuroDNS: registration/renewal `€353.50`/year; no registration restrictions.

### Availability check

Direct WHOIS query to `whois.nic.md` returned:

```text
Domain name:   tandem.md
Domain state:  OK
Registered on: 2008-10-15
Expires on:    2026-10-15
NameServer:    johnny.ns.cloudflare.com
NameServer:    paislee.ns.cloudflare.com
```

DNS also resolves for `tandem.md` with Cloudflare nameservers and A records. This strongly indicates it is active, not available for ordinary registration.

### Alternatives checked quickly

Direct `.md` WHOIS checks returned no match for these examples:

- `gettandem.md`
- `tandemdash.md`
- `tandemdocs.md`
- `tandemprotocol.md`

Quick DNS checks suggest these exact non-`.md` options are already registered or delegated:

- `tandem.dev` — GoDaddy nameservers / parked-style A records.
- `tandem.sh` — GoDaddy nameservers / parked-style A records.
- `tandem.app` — Cloudflare nameservers / A records.

These DNS checks are not a substitute for registrar checkout/WHOIS/RDAP, but they are enough to treat the exact names as likely unavailable.

## Eligibility, trademark, and policy notes

- Eligibility: no local Moldova presence requirement was found; NIC.MD and EuroDNS both indicate open registration.
- Trademark: open registration does not remove trademark risk. `Tandem` is a common product/company term, so do a basic trademark and search-engine clearance before using a domain for a public product. If buying `tandem.md` from its current registrant, also verify there are no conflicting marks, prior use claims, or transfer restrictions.
- Renewal/expiry: registrars may add their own grace/redemption policies. Gandi states `.md` domains are deactivated on expiration and can be renewed up to 45 days after expiration, with deletion/quarantine also noted as 45 days. Do not rely on the October 2026 expiration date as an acquisition strategy.

## DNS/hosting setup if an alternative is registered

1. Register the selected domain at NIC.MD or a registrar with acceptable pricing/renewal terms.
2. Choose DNS provider:
   - Registrar DNS is simplest.
   - Cloudflare is a good default if you want proxying, redirects, DNSSEC/UI, and separate hosting.
   - Vercel/GitHub Pages DNS works well if the docs site is hosted there.
3. Configure records for the hosting target:
   - Apex domain usually needs `A`, `AAAA`, `ALIAS`, or `ANAME` depending on host/DNS provider.
   - `www` or `docs` subdomains usually use `CNAME`.
   - Add TXT verification records if the host requires domain verification.
4. Enable HTTPS and verify redirects between apex and `www`/canonical docs URL.
5. If using GitHub Pages, verify the domain in GitHub to reduce custom-domain takeover risk.

## Practical options

1. **Register `gettandem.md` now** — best `.md`-themed option found in this pass; likely available, memorable, and directly tied to the desired Markdown/domain pun.
2. **Use a docs subdomain on another owned domain** — lowest risk and fastest if there is already a canonical organization/project domain.
3. **Use a developer TLD with a modifier** — e.g. `gettandem.dev`, `tandemdocs.dev`, `tandemprotocol.dev`, or similar after registrar checks.
4. **Acquire `tandem.md`** — only if exact branding matters enough to justify broker/outreach cost and uncertainty.

## Sources checked

- IANA WHOIS for `.md` delegation via direct WHOIS query to `whois.iana.org`.
- NIC.MD official site and WHOIS: https://nic.md/en/ and https://nic.md/en/whois/
- TLD-List `.md` price comparison search result: https://tld-list.com/tld/md
- Gandi `.md` page: https://www.gandi.net/en-US/domain/tld/md
- EuroDNS `.md` page: https://www.eurodns.com/domain-extensions/md-domain-registration
- Cloudflare domain onboarding docs: https://developers.cloudflare.com/fundamentals/manage-domains/add-site/
- GitHub Pages custom domain docs: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/about-custom-domains-and-github-pages
- Vercel custom domain docs: https://vercel.com/docs/domains/working-with-domains/add-a-domain

## Validation commands run

```sh
python3 - <<'PY'
import socket
for host in ['whois.nic.md','whois.iana.org']:
    s = socket.create_connection((host, 43), timeout=10)
    s.sendall(b'tandem.md\r\n')
    print(s.recv(8192).decode('utf-8', 'ignore'))
    s.close()
PY

for d in tandem.md gettandem.md tandem.dev tandem.sh tandem.app tandemdash.md tandemdocs.md tandemprotocol.md; do
  dig +short NS "$d"
  dig +short A "$d"
done
```
