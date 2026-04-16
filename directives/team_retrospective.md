# 🔄 Directive: Team Retrospective (SOP-OPS-04)

## 🎯 Primary Objective
Audit the previous sprint's execution to identify architectural friction and operational inefficiencies. The goal is to "Self-Anneal" the swarm's performance.

---

## 🏗️ Retrospective Scope

### 1. The "What Went Well" Analysis
- **Action**: Identify the top 3 mission victories. Which agents showed the highest "Intelligence-per-Token"?
- **Wisdom**: Extract the patterns used by these agents into `LONG_TERM_MEMORY.md`.

### 2. Incident Review
- **Source**: `incident_response.md` logs.
- **Action**: Analyze any system crashes or budget overruns. Was the root cause technical (Rust fault) or instructional (Directive ambiguity)?

### 3. Tool Friction
- **Focus**: Did any `execution/*.py` scripts cause repeated failures? 
- **Action**: Flag for refactoring in the next `product_sync.md`.

---

## 🛠️ Optimization SOP

### 1. Process Update
- **Action**: Refine the `directives/` SOPs based on "Lessons Learned" during the sprint.

### 2. Team Health (System Pulse)
- **Check**: Audit `failure_count` in the `agents` registry. Any agent exceeding 10 total failures requires a "Neuro-Reconfiguration" (Role/Model update).

---

## 📝 Reporting Protocol
The final `RETROSPECTIVE_[ID].md` must be archived in the `memory.lance` vector store to ensure future swarms don't repeat historical errors.