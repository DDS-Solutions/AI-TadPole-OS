> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / README
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[README]`)

# Tadpole OS - Mobile Companion App (Android)

> **Design Alignment**: Built in strict adherence to the [Tadpole OS Sovereign Design Contract (`DESIGN.md`)](../../DESIGN.md).

---

## 🎨 Mobile Visual Identity & Design Tokens

The Android companion app carries over the exact design aesthetic, HSL-tinted obsidian dark mode, and color palette from the main desktop dashboard:

| Design Token | Color Hex | Use Case in Mobile App |
|---|---|---|
| **Obsidian Background** | `#090D16` | Main screen background (`background`) |
| **Surface Layer 1** | `#111827` / `#1E293B` | Approval Cards, Health Grid Cards, Card Containers |
| **Emerald Accent** | `#10B981` | "Sign & Approve" button, active/running status indicators |
| **Cyan Accent** | `#06B6D4` | Agent Health panel icons, active telemetry counters |
| **Amber Warning** | `#F59E0B` | Halted agent state, pending approval warning badges |
| **Red Alert** | `#DC2626` / `#EF4444` | Emergency Swarm Panic Freeze Switch, reject buttons |

---

## 🚫 Mobile Anti-Patterns Checklist (from `DESIGN.md`)

1. **No Pure `#000000`**: Always use deep HSL-tinted obsidian (`#090D16`).
2. **No Card Inception**: Avoid nesting cards inside cards with duplicate borders.
3. **Monospace for Technical Metrics**: Use `FontFamily.Monospace` for trace IDs, timestamps, and tool names.
4. **Accessible Contrast**: Ensure text contrast on status badges (`#10B981` text on dark emerald tint background).

---

## 🔐 Remote Oversight & Hardware Biometric Architecture

- **QR Pairing & Ed25519 Identity**: Pair with a desktop node via single-use QR challenges (`POST /v1/remote/pair`; pairing tokens require `RequireAdmin` / `NEURAL_ADMIN_TOKEN` on desktop). Authenticated requests sign canonical `METHOD:PATH:TIMESTAMP:NONCE` headers (`X-Device-Id`, `X-Timestamp`, `X-Nonce`, `X-Signature`).
- **Hardware-Backed Biometric Decisions**: `BiometricSignatureManager.kt` wraps the companion Ed25519 private key with an `AndroidKeyStore` **RSA-2048 OAEP** keypair (`setUserAuthenticationRequired(true)`, `setInvalidatedByBiometricEnrollment(true)`, `BIOMETRIC_STRONG` only — no device-credential fallback). `promptBiometricDecision()` requires a hardware-authenticated `BiometricPrompt.CryptoObject(cipher)` unwrap, wipes decrypted key bytes in `finally`, and recovers cleanly from `KeyPermanentlyInvalidatedException`.

---

## 🚀 Building, Testing & Opening in Android Studio

### Requirements
- **JDK 21**, **Android SDK** (`compileSdk = 34`, `minSdk = 26`, `targetSdk = 34`), Android Studio (Iguana / Jellyfish or newer).

### CLI Build & Unit Tests
```bash
cd apps/mobile-android
./gradlew testDebugUnitTest
./gradlew assembleDebug
```

### Android Studio
1. Launch **Android Studio**.
2. Select `File -> Open` and choose `apps/mobile-android`.
3. Gradle will sync dependencies automatically.