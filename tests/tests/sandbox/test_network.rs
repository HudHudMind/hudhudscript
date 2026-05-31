//! Tests extracted from hudhudscript-sandbox/src/network.rs

use hudhudscript_sandbox::{NetworkConfig, NetworkSandbox};

#[test]
fn test_domain_access() {
    let config = NetworkConfig {
        allow_domains: vec!["api.example.com".to_string(), "*.trusted.com".to_string()],
        allow_ports: vec![80, 443],
        deny_hosts: vec!["localhost".to_string()],
    };

    let sandbox = NetworkSandbox::new(config);

    // Should allow access to allowed domains
    assert!(sandbox.check_access("api.example.com", 443).is_ok());
    assert!(sandbox.check_access("sub.trusted.com", 443).is_ok());

    // Should deny access to localhost
    assert!(sandbox.check_access("localhost", 8080).is_err());

    // Should deny access to non-allowed domains
    assert!(sandbox.check_access("evil.com", 443).is_err());
}

#[test]
fn test_port_restrictions() {
    let config = NetworkConfig {
        allow_domains: vec!["api.example.com".to_string()],
        allow_ports: vec![443],
        deny_hosts: vec![],
    };

    let sandbox = NetworkSandbox::new(config);

    // Should allow allowed port
    assert!(sandbox.check_access("api.example.com", 443).is_ok());

    // Should deny non-allowed port
    assert!(sandbox.check_access("api.example.com", 8080).is_err());
}

#[test]
fn test_wildcard_domains() {
    let config = NetworkConfig {
        allow_domains: vec!["*.example.com".to_string()],
        allow_ports: vec![],
        deny_hosts: vec![],
    };

    let sandbox = NetworkSandbox::new(config);

    // Should match wildcard subdomains
    assert!(sandbox.check_access("api.example.com", 443).is_ok());
    assert!(sandbox.check_access("www.example.com", 80).is_ok());

    // Should match base domain
    assert!(sandbox.check_access("example.com", 443).is_ok());

    // Should not match different domain
    assert!(sandbox.check_access("example.org", 443).is_err());

    // Should NOT match a domain that merely ends with same suffix but different boundary
    assert!(sandbox.check_access("notexample.com", 443).is_err());
}

#[test]
fn test_ip_wildcard_pattern() {
    let config = NetworkConfig {
        allow_domains: vec!["192.168.*".to_string()],
        allow_ports: vec![],
        deny_hosts: vec![],
    };
    let sandbox = NetworkSandbox::new(config);
    assert!(sandbox.check_access("192.168.1.1", 80).is_ok());
    assert!(sandbox.check_access("192.168.0.1", 443).is_ok());
    assert!(sandbox.check_access("10.0.0.1", 80).is_err());
}

#[test]
fn test_exact_host_match() {
    let config = NetworkConfig {
        allow_domains: vec!["api.example.com".to_string()],
        allow_ports: vec![],
        deny_hosts: vec![],
    };
    let sandbox = NetworkSandbox::new(config);
    assert!(sandbox.check_access("api.example.com", 443).is_ok());
    assert!(sandbox.check_access("other.example.com", 443).is_err());
}

#[test]
fn test_empty_port_list_allows_all() {
    let config = NetworkConfig {
        allow_domains: vec!["api.example.com".to_string()],
        allow_ports: vec![], // Empty means all ports allowed
        deny_hosts: vec![],
    };
    let sandbox = NetworkSandbox::new(config);
    assert!(sandbox.check_access("api.example.com", 9999).is_ok());
}

#[test]
fn test_ipv6_loopback_denied() {
    let config = NetworkConfig {
        allow_domains: vec!["*".to_string()],
        allow_ports: vec![],
        deny_hosts: vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ],
    };

    let sandbox = NetworkSandbox::new(config);

    assert!(sandbox.check_access("::1", 8080).is_err());
    assert!(sandbox.check_access("127.0.0.1", 3000).is_err());
    assert!(sandbox.check_access("localhost", 80).is_err());
    // Other hosts should be allowed
    assert!(sandbox.check_access("api.example.com", 443).is_ok());
}
