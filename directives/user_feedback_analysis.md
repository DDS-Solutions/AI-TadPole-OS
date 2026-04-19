# 🗣️ Directive: User Feedback Analysis (SOP-MKT-06)

## 🎯 Primary Objective
Transform raw user feedback into structured product intelligence. This directive ensures that the "Voice of the Customer" is a first-class citizen in the sprint planning process.

---

## 🏗️ Analysis Workflow

### 1. Ingestion
- **Sources**: GitHub Issues, Discord technical support threads, and direct outreach.
- **Action**: Aggregate text data into a temporary `feedback_buffer.txt`.

### 2. Categorization
- **Categories**: 
  - **Feature Request**: New functionality.
  - **Bug Report**: Regression or failure.
  - **UI/UX**: Friction in the dashboard.
  - **Documentation**: Missing or unclear SOPs.

### 3. Sentiment Scoring
- **Action**: Use a local LLM to score each entry from -1.0 (Critical Frustration) to +1.0 (Evangelist).
- **Metric**: Calculate "Average Sentiment Velocity" over a 7-day window.

---

## 📈 Prioritization (The Quadrant)
- **High Impact / High Sentiment**: "Love the feature, please expand." (Focus here for growth).
- **High Impact / Low Sentiment**: "System crashed during mission." (Focus here for stability).

---

## 📊 Output Protocol
Report monthly findings in `reports/USER_FEEDBACK_[MONTH].md`. Include a "Top 3 Actionable Fixes" list for the next `product_sync.md`.

## 🚦 Quality Gate
Ensure and distinguish between "Self-Selection Bias" (vocal minority) and "General User Friction" (data-driven patterns).