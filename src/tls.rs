//! TLS backend selection and `rustls` crypto-provider installation.
//!
//! `deribit-websocket` supports three mutually-exclusive TLS backends,
//! selected at compile time via Cargo features (see this crate's
//! `Cargo.toml` or the README):
//!
//! - `rustls-aws-lc` (default) — `rustls` with the `aws-lc-rs` crypto
//!   provider, OS-native root store.
//! - `rustls-ring` — `rustls` with the `ring` crypto provider, OS-native
//!   root store.
//! - `native-tls` — OS-native TLS stack (SChannel / SecureTransport /
//!   OpenSSL); no `rustls` dependency is pulled in.
//!
//! Exactly one of the three must be active. The [`compile_error!`]
//! gates below reject any other combination — zero or two or three — at
//! compile time so misconfigured builds fail fast rather than at
//! runtime during the first WebSocket handshake.
//!
//! # Crypto provider
//!
//! The two `rustls-*` backends rely on `rustls`'s process-global crypto
//! provider slot being populated before any TLS handshake starts.
//! Applications must call [`install_default_crypto_provider`] once at
//! startup; the helper picks the right provider for the active feature
//! and is a no-op under `native-tls`.

// ---------------------------------------------------------------------
// Compile-time mutex — exactly one TLS backend.
// ---------------------------------------------------------------------

#[cfg(not(any(
    feature = "rustls-aws-lc",
    feature = "rustls-ring",
    feature = "native-tls"
)))]
compile_error!(
    "deribit-websocket: select one of the TLS backends via Cargo features: \
     `rustls-aws-lc`, `rustls-ring`, or `native-tls`"
);

#[cfg(any(
    all(feature = "rustls-aws-lc", feature = "rustls-ring"),
    all(feature = "rustls-aws-lc", feature = "native-tls"),
    all(feature = "rustls-ring", feature = "native-tls"),
))]
compile_error!(
    "deribit-websocket: select exactly one TLS backend; the features \
     `rustls-aws-lc`, `rustls-ring`, and `native-tls` are mutually exclusive"
);

// ---------------------------------------------------------------------
// Public API — crypto-provider installation.
// ---------------------------------------------------------------------

/// Errors raised when installing the `rustls` crypto provider.
///
/// Separate from [`crate::error::WebSocketError`] because this is a
/// one-shot startup concern distinct from runtime protocol errors;
/// folding it into `WebSocketError` would bloat the main enum for a
/// call that runs once per process.
#[derive(Debug, thiserror::Error)]
pub enum CryptoProviderError {
    /// A `rustls` crypto provider is already installed in this process.
    ///
    /// `rustls` stores its crypto provider in a process-global
    /// `OnceCell`-style slot; subsequent `install_default` calls are
    /// rejected. This variant carries no payload because the already-
    /// installed provider is rarely actionable from the caller, and
    /// exposing an `Arc<CryptoProvider>` here would leak a `rustls`
    /// type into callers that are compiled under `native-tls`.
    #[error("a rustls crypto provider is already installed in this process")]
    AlreadyInstalled,
}

/// Install the process-global `rustls` crypto provider matching the
/// active TLS feature.
///
/// - Under `rustls-aws-lc`, installs
///   `rustls::crypto::aws_lc_rs::default_provider()`.
/// - Under `rustls-ring`, installs
///   `rustls::crypto::ring::default_provider()`.
/// - Under `native-tls`, this is a no-op that returns `Ok(())` — the
///   OS TLS stack does not require any process-level initialization.
///
/// Call this exactly once at application startup, before any call to
/// [`crate::DeribitWebSocketClient::connect`]. Subsequent calls return
/// [`CryptoProviderError::AlreadyInstalled`] rather than panic, which
/// lets callers be robust against multiple entry points (tests, libs,
/// `main`) all trying to initialize the provider.
///
/// # Errors
///
/// Returns [`CryptoProviderError::AlreadyInstalled`] if a provider is
/// already installed in this process by this call, a previous call, or
/// any other library that uses `rustls`.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Idempotent: subsequent calls return AlreadyInstalled but are
/// // otherwise safe.
/// let _ = deribit_websocket::install_default_crypto_provider();
/// # Ok(())
/// # }
/// ```
// `Result` already carries `#[must_use]`, so an extra attribute on the
// function itself would be redundant (see `clippy::double_must_use`).
#[inline(never)]
// The explicit `return`s below are load-bearing: without them the
// function body would have multiple `#[cfg]`-gated tail expressions and
// fail to type-check when more than one TLS feature is active — the
// very case the `compile_error!` mutex above is meant to reject
// cleanly. Allowing `needless_return` keeps the diagnostic limited to
// the mutex message alone.
#[allow(clippy::needless_return)]
pub fn install_default_crypto_provider() -> Result<(), CryptoProviderError> {
    // The compile-time mutex above guarantees exactly one of the three
    // branches below is live; the priority order (aws-lc > ring >
    // native-tls) only matters when the mutex has already triggered —
    // making each branch mutually exclusive via `not(...)` guards keeps
    // the function body type-correct under any feature combination so
    // the build output contains only the mutex diagnostic, not
    // cascading `E0308` noise.
    #[cfg(feature = "rustls-aws-lc")]
    {
        return rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .map_err(|_| CryptoProviderError::AlreadyInstalled);
    }
    #[cfg(all(feature = "rustls-ring", not(feature = "rustls-aws-lc")))]
    {
        return rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| CryptoProviderError::AlreadyInstalled);
    }
    #[cfg(all(
        feature = "native-tls",
        not(feature = "rustls-aws-lc"),
        not(feature = "rustls-ring")
    ))]
    {
        return Ok(());
    }

    // Fallback for the "no TLS feature selected" case: the
    // `compile_error!` at the top of this module has already fired, so
    // this path is unreachable in any valid build. Returning a typed
    // `Ok(())` keeps `rustc` from piling a secondary `E0308` on top of
    // the real diagnostic.
    #[cfg(not(any(
        feature = "rustls-aws-lc",
        feature = "rustls-ring",
        feature = "native-tls"
    )))]
    #[allow(unreachable_code)]
    Ok(())
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::install_default_crypto_provider;

    #[cfg(any(feature = "rustls-aws-lc", feature = "rustls-ring"))]
    use super::CryptoProviderError;

    /// Under `native-tls` the helper is a no-op and always succeeds.
    /// Under the `rustls-*` backends the *first* install may race with
    /// other tests, so we don't assert its result; instead we
    /// guarantee that a *second* call deterministically returns
    /// `AlreadyInstalled`.
    #[test]
    fn second_install_is_deterministic() {
        // Prime the slot. Outcome does not matter — another test in the
        // same process may have installed already.
        let _ = install_default_crypto_provider();

        #[cfg(any(feature = "rustls-aws-lc", feature = "rustls-ring"))]
        {
            match install_default_crypto_provider() {
                Err(CryptoProviderError::AlreadyInstalled) => {}
                other => panic!("expected AlreadyInstalled, got {other:?}"),
            }
        }

        #[cfg(feature = "native-tls")]
        {
            // Under native-tls every call is Ok(()).
            match install_default_crypto_provider() {
                Ok(()) => {}
                other => panic!("expected Ok under native-tls, got {other:?}"),
            }
        }
    }
}
