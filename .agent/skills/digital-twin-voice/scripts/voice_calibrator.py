#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / digital-twin-voice / voice_calibrator
- **Primary Entrypoints**: `analyze_text_samples`, `extract_lexical_patterns`, `generate_persona_artifact`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic tone metric scoring and persona artifact generation without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[voice_calibrator]`, `[PASS]`, `[FAIL]`, `[VOICE]`, `[CALIBRATE]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional, Tuple

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


INFORMAL_MARKERS = {
    "hey", "hi", "hello", "cheers", "cool", "awesome", "yeah", "yep", "nope",
    "gonna", "wanna", "gotta", "asap", "fyi", "btw", "thanks", "thx", "pls", "please"
}

FORMAL_MARKERS = {
    "furthermore", "moreover", "regarding", "consequently", "sincerely",
    "respectfully", "cordially", "herewith", "aforementioned", "subsequently",
    "pursuant", "endeavor", "facilitate", "inquire", "commence"
}

ENTHUSIASM_MARKERS = {
    "great", "amazing", "fantastic", "wonderful", "love", "thrilled", "excited",
    "super", "perfect", "brilliant", "delighted", "appreciate", "outstanding"
}


def extract_lexical_patterns(samples: List[str]) -> Dict[str, Any]:
    """
    Extract repeated greetings, sign-offs, and frequent phrasing from communication samples.
    """
    greetings = []
    signoffs = []
    
    greeting_pattern = re.compile(r'^(?:Hey|Hi|Hello|Good morning|Good afternoon|Dear)\b[^\n,]*', re.IGNORECASE | re.MULTILINE)
    signoff_pattern = re.compile(r'(?:Best regards|Warm regards|Best|Cheers|Thanks|Thank you|Sincerely|Talk soon)[,\s]*\n?[A-Za-z\s]*$', re.IGNORECASE | re.MULTILINE)
    
    for sample in samples:
        for m in greeting_pattern.finditer(sample):
            greetings.append(m.group(0).strip())
        for m in signoff_pattern.finditer(sample):
            signoffs.append(m.group(0).strip())
            
    # Count frequencies
    greeting_counts: Dict[str, int] = {}
    for g in greetings:
        norm = g.title()
        greeting_counts[norm] = greeting_counts.get(norm, 0) + 1
        
    signoff_counts: Dict[str, int] = {}
    for s in signoffs:
        norm = s.strip(',').title()
        signoff_counts[norm] = signoff_counts.get(norm, 0) + 1
        
    top_greetings = sorted(greeting_counts.items(), key=lambda x: x[1], reverse=True)
    top_signoffs = sorted(signoff_counts.items(), key=lambda x: x[1], reverse=True)
    
    return {
        "preferred_greetings": [g[0] for g in top_greetings[:3]],
        "preferred_signoffs": [s[0] for s in top_signoffs[:3]]
    }


def analyze_text_samples(samples: List[str]) -> Dict[str, Any]:
    """
    Analyze text samples along key tone dimensions: Formality, Enthusiasm, Directness.
    Returns normalized scores from 0.0 to 1.0.
    """
    if not samples:
        return {
            "formality": 0.5,
            "enthusiasm": 0.5,
            "directness": 0.5,
            "word_count": 0,
            "sentence_count": 0,
            "lexical": {"preferred_greetings": [], "preferred_signoffs": []}
        }
        
    full_text = " \n".join(samples)
    words = re.findall(r'\b[a-zA-Z\']+\b', full_text.lower())
    total_words = len(words) or 1
    
    sentences = re.split(r'[.!?]+', full_text)
    sentences = [s.strip() for s in sentences if s.strip()]
    total_sentences = len(sentences) or 1
    
    # 1. Formality calculation
    informal_count = sum(1 for w in words if w in INFORMAL_MARKERS)
    formal_count = sum(1 for w in words if w in FORMAL_MARKERS)
    contraction_count = len(re.findall(r"\b\w+'(?:t|re|ve|m|ll|d|s)\b", full_text.lower()))
    
    # More contractions + informal markers = lower formality
    informality_signal = (informal_count * 2 + contraction_count) / total_words
    formal_signal = (formal_count * 4) / total_words
    formality_score = 0.5 + (formal_signal - informality_signal) * 5.0
    formality_score = max(0.05, min(0.95, round(formality_score, 2)))
    
    # 2. Enthusiasm calculation
    exclamation_count = full_text.count('!')
    enthusiasm_word_count = sum(1 for w in words if w in ENTHUSIASM_MARKERS)
    emoji_count = len(re.findall(r'[\U00010000-\U0010ffff]', full_text))
    
    enthusiasm_signal = (exclamation_count * 2 + enthusiasm_word_count * 1.5 + emoji_count * 3) / total_sentences
    enthusiasm_score = 0.3 + (enthusiasm_signal * 0.4)
    enthusiasm_score = max(0.05, min(0.95, round(enthusiasm_score, 2)))
    
    # 3. Directness calculation (Mean sentence length)
    mean_sentence_length = total_words / total_sentences
    # Short sentences (< 12 words) = high directness (concise); Long sentences (> 25 words) = low directness (verbose/storytelling)
    if mean_sentence_length <= 10:
        directness_score = 0.85
    elif mean_sentence_length <= 16:
        directness_score = 0.65
    elif mean_sentence_length <= 24:
        directness_score = 0.45
    else:
        directness_score = 0.25
        
    lexical = extract_lexical_patterns(samples)
    
    return {
        "formality": formality_score,
        "enthusiasm": enthusiasm_score,
        "directness": directness_score,
        "word_count": total_words,
        "sentence_count": total_sentences,
        "mean_sentence_length": round(mean_sentence_length, 1),
        "lexical": lexical
    }


def generate_persona_artifact(
    profile: Dict[str, Any],
    brand_name: str = "SMB Founder",
    target_path: Optional[Path] = None
) -> str:
    """
    Generate calibrated Digital Twin system prompt artifact in markdown format.
    """
    formality_label = (
        "Highly Formal & Academic" if profile["formality"] > 0.75
        else "Professional & Polished" if profile["formality"] > 0.55
        else "Casual & Conversational" if profile["formality"] > 0.35
        else "Ultra-Casual & Laid-back"
    )
    
    enthusiasm_label = (
        "High Energy & Vibrant" if profile["enthusiasm"] > 0.7
        else "Warm & Encouraging" if profile["enthusiasm"] > 0.45
        else "Calm, Measured & Pragmatic"
    )
    
    directness_label = (
        "Direct, Punchy & Bullet-Driven" if profile["directness"] > 0.65
        else "Balanced Explanations" if profile["directness"] > 0.4
        else "Elaborative & Narrative-Driven"
    )
    
    greetings = ", ".join(f'"{g}"' for g in profile["lexical"]["preferred_greetings"]) or '"Hi there", "Hello"'
    signoffs = ", ".join(f'"{s}"' for s in profile["lexical"]["preferred_signoffs"]) or '"Best", "Cheers"'
    
    artifact = f"""# Digital Twin Voice Profile — {brand_name}
> Calibrated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} via `voice_calibrator.py`.

---

## 🎯 Core Persona Dimensions

- **Formality ({profile['formality']}/1.0)**: {formality_label}
- **Enthusiasm ({profile['enthusiasm']}/1.0)**: {enthusiasm_label}
- **Directness ({profile['directness']}/1.0)**: {directness_label}
- **Average Sentence Length**: {profile.get('mean_sentence_length', 14)} words/sentence

---

## 🗣️ Signature Vocabulary & Phrasing

- **Preferred Greetings**: {greetings}
- **Preferred Sign-offs**: {signoffs}
- **Tone Directive**: Speak with the natural voice of {brand_name}. Avoid corporate boilerplate, unearned jargon, and sterile robotic greetings.

---

## 🚫 Persona Guardrails & Negative Constraints

1. **Never Sound Like Generic AI**: Avoid "As an AI language model...", "I hope this email finds you well", or "Please let me know if you need any further assistance!".
2. **Honor Directness Budget**: Keep answers within {profile.get('mean_sentence_length', 14) * 5} words unless deep analytical detail is explicitly requested.
3. **Redact Sensitive PII**: All customer credit cards, SSNs, and personal addresses must remain strictly redacted per `@[skills/pii-redaction]`.
"""
    if target_path:
        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_text(artifact, encoding='utf-8')
        
    return artifact


def run_self_tests() -> bool:
    """
    Execute self-tests verifying tone scoring, lexical extraction, and persona artifact generation.
    """
    print("[voice_calibrator] Executing self-test suite...")
    
    # 1. Casual sample
    casual_samples = [
        "Hey team! Super excited about the launch. Can we get this done asap? Cheers, Alex",
        "Hi John, yeah that sounds awesome! Let's do it. Thanks, Alex"
    ]
    casual_profile = analyze_text_samples(casual_samples)
    assert casual_profile["formality"] < 0.5, f"Expected casual text to have formality < 0.5, got {casual_profile['formality']}"
    assert casual_profile["enthusiasm"] >= 0.4, f"Expected enthusiastic text to have enthusiasm >= 0.4, got {casual_profile['enthusiasm']}"
    assert "Hey" in casual_profile["lexical"]["preferred_greetings"] or "Hi John" in casual_profile["lexical"]["preferred_greetings"]
    
    # 2. Formal sample
    formal_samples = [
        "Dear Mr. Smith, Regarding the aforementioned agreement, we consequently commence review. Sincerely, Executive Office",
        "Dear Board of Directors, Furthermore, we endeavor to facilitate all compliance operations. Respectfully, Executive Office"
    ]
    formal_profile = analyze_text_samples(formal_samples)
    assert formal_profile["formality"] > casual_profile["formality"], "Expected formal text to score higher formality than casual text"
    
    # 3. Artifact generation
    artifact = generate_persona_artifact(casual_profile, brand_name="Tadpole Bakery")
    assert "# Digital Twin Voice Profile — Tadpole Bakery" in artifact
    assert "Preferred Greetings" in artifact
    assert "Persona Guardrails" in artifact
    
    print("[voice_calibrator] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic Digital Twin Voice Calibrator")
    parser.add_argument("path", nargs="?", default=None, help="Path to sample file or directory containing communications (.txt, .md, .json)")
    parser.add_argument("--test", action="store_true", help="Execute self-test suite and exit")
    parser.add_argument("--text", type=str, default=None, help="Direct inline text sample to analyze")
    parser.add_argument("--brand", type=str, default="SMB Founder", help="Brand or founder name")
    parser.add_argument("--output", type=str, default=None, help="Output markdown path for calibrated persona (default: .tmp/twin-persona.md)")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON profile")
    
    args = parser.parse_args()
    
    if args.test:
        try:
            success = run_self_tests()
            sys.exit(0 if success else 1)
        except AssertionError as e:
            print(f"[voice_calibrator] [FAIL] Self-test assertion failed: {e}")
            sys.exit(1)
            
    samples: List[str] = []
    
    if args.text:
        samples.append(args.text)
        
    if args.path:
        p = Path(args.path).resolve()
        if p.is_file():
            try:
                samples.append(p.read_text(encoding='utf-8', errors='ignore'))
            except Exception as e:
                print(f"[voice_calibrator] [FAIL] Error reading file {p}: {e}")
                sys.exit(1)
        elif p.is_dir():
            for f in p.glob('**/*'):
                if f.is_file() and f.suffix.lower() in ('.txt', '.md', '.log'):
                    try:
                        samples.append(f.read_text(encoding='utf-8', errors='ignore'))
                    except Exception:
                        pass
                        
    if not samples:
        print("[voice_calibrator] [VOICE] No text samples provided. Use --text or provide a file path.")
        sys.exit(0)
        
    print("[voice_calibrator] Initializing voice calibration...")
    print("[VOICE] Ingesting communication samples...")
    print(f"[CALIBRATE] Extracting tone dimensions for brand '{args.brand}'...")
    
    profile = analyze_text_samples(samples)
    
    out_path = Path(args.output) if args.output else Path(".tmp/twin-persona.md")
    artifact = generate_persona_artifact(profile, brand_name=args.brand, target_path=out_path)
    
    if args.json:
        result = {
            "script": "voice_calibrator",
            "brand": args.brand,
            "profile": profile,
            "artifact_saved_to": str(out_path) if args.output or not args.json else None
        }
        print(json.dumps(result, indent=2))
        sys.exit(0)
        
    print(f"\n{'='*64}")
    print(f"🎙️ [VOICE CALIBRATOR] Calibrated Persona Profile")
    print(f"{'='*64}")
    print(f"Brand / Founder   : {args.brand}")
    print(f"Total Words       : {profile['word_count']}")
    print(f"Total Sentences   : {profile['sentence_count']}")
    print(f"Mean Sentence Len : {profile['mean_sentence_length']} words")
    print(f"{'-'*64}")
    print(f"📊 Formality      : {profile['formality']} / 1.0")
    print(f"🔥 Enthusiasm     : {profile['enthusiasm']} / 1.0")
    print(f"🎯 Directness     : {profile['directness']} / 1.0")
    print(f"👋 Greetings      : {', '.join(profile['lexical']['preferred_greetings']) or 'None'}")
    print(f"✍️  Sign-offs      : {', '.join(profile['lexical']['preferred_signoffs']) or 'None'}")
    print(f"{'-'*64}")
    print(f"💾 Persona Artifact Generated: {out_path}")
    print("[voice_calibrator] [PASS] Voice calibration completed successfully.")
    print(f"{'='*64}\n")
    
    sys.exit(0)


if __name__ == "__main__":
    main()
