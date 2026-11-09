> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tdd-workflow
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Writing production code before failing test assertions or skipping refactoring.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[TDD_WORKFLOW]`)

# Test-Driven Development (TDD) Cycles & Anti-Patterns (L3)

---

## 1. Concrete RED-GREEN-REFACTOR Cycle

### 🔴 RED Phase: Write Failing Test & Capture Assertion Witness
```typescript
it('calculates capability token expiration timestamp', () => {
  const expiry = computeExpiry(1000, 3600);
  expect(expiry).toBe(4600);
});
```
*Witness Proof*: Execute `npm test` and verify that the failure is an assertion error (`AssertionError: expected undefined to be 4600`), NOT a syntax or import failure.

### 🟢 GREEN Phase: Minimal Implementation
```typescript
export function computeExpiry(issuedAt: number, ttlSecs: number): number {
  return issuedAt + ttlSecs;
}
```
*Witness Proof*: Execute `npm test` and verify test passes with exit code 0.

### 🔵 REFACTOR Phase: Clean & Harden
```typescript
export function computeExpiry(issuedAt: number, ttlSecs: number): number {
  if (issuedAt < 0 || ttlSecs <= 0) throw new RangeError('Timestamps must be positive');
  return issuedAt + ttlSecs;
}
```

---

## 2. Verification Before Completion Protocol

Before declaring an implementation task complete, run the following verification pipeline:
1. `git diff tests/`: Verify the newly written test covers the core user requirement.
2. `npm test` or `cargo test`: Ensure all test suites pass with 100% green exit code.
3. `git status`: Ensure no temporary test artifacts (`.tmp/`, `.log`) remain untracked.

---

## 3. The Three Laws of TDD

1. **Law 1**: You may not write production code until you have written a failing unit test.
2. **Law 2**: You may not write more of a unit test than is sufficient to fail.
3. **Law 3**: You may not write more production code than is sufficient to pass the failing test.