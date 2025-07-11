# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| 0.9.x   | :white_check_mark: |
| 0.8.x   | :x:                |
| < 0.8   | :x:                |

## Reporting a Vulnerability

We take the security of Canvas Contracts seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to [security@canvascontracts.com](mailto:security@canvascontracts.com).

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the requested information listed below (as much as you can provide) to help us better understand the nature and scope of the possible issue:

- Type of issue (buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the vulnerability
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Preferred Languages

We prefer all communications to be in English.

## Policy

Canvas Contracts follows the principle of [Responsible Disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure).

## Security Best Practices

When using Canvas Contracts, please follow these security best practices:

1. **Keep Dependencies Updated**: Regularly update your dependencies to the latest secure versions
2. **Validate Input**: Always validate and sanitize input data in your smart contracts
3. **Use Secure Randomness**: Use cryptographically secure random number generators
4. **Implement Access Controls**: Properly implement access controls and permissions
5. **Test Thoroughly**: Test your contracts extensively before deployment
6. **Audit Code**: Consider having your contracts audited by security professionals
7. **Monitor Deployments**: Monitor your deployed contracts for suspicious activity

## Security Features

Canvas Contracts includes several security features:

- **Static Analysis**: Built-in static analysis tools to detect common vulnerabilities
- **Gas Optimization**: Automatic gas optimization to prevent DoS attacks
- **Type Safety**: Strong type checking to prevent runtime errors
- **Sandboxed Execution**: WASM execution in a secure sandbox environment
- **Formal Verification**: Support for formal verification of contract logic

## Responsible Disclosure Timeline

- **Day 0**: Vulnerability reported
- **Day 1**: Acknowledgment of receipt
- **Day 7**: Initial assessment and response
- **Day 30**: Status update and timeline for fix
- **Day 90**: Public disclosure (if not fixed)

## Recognition

We recognize security researchers who responsibly disclose vulnerabilities by:

- Adding them to our [Security Hall of Fame](SECURITY_HALL_OF_FAME.md)
- Mentioning them in release notes
- Providing appropriate attribution in security advisories

## Contact

For security-related questions or concerns, please contact us at [security@canvascontracts.com](mailto:security@canvascontracts.com).

---

Thank you for helping keep Canvas Contracts secure! 