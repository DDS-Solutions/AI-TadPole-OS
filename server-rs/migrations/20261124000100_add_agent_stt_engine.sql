-- Persist per-agent speech-to-text engine selection (groq | whisper)
ALTER TABLE agents ADD COLUMN stt_engine TEXT;
