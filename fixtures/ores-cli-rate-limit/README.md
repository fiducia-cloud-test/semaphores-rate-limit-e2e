# ORES CLI rate-limit runtime-evidence corpus

These fixtures keep the same strict Redis-oriented `.ores-rl.toml` claim while varying only the repository's executable evidence. They are intended for external `ORESoftware/ores-cli` acceptance testing of runtime-consumption classification.

The local validator checks fixture intent only; it does not duplicate ores-cli's classifier. A private ores-cli acceptance lane can pin this public test-org commit and run the real binary over each case.
