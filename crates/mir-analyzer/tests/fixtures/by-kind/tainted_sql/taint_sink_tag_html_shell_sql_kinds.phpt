===description===
@taint-sink's free-text kind now recognizes html/sql/shell (not just
llm_prompt), reusing the same issue their built-in-function sinks raise.
Each is on its own function so the location pinpoints its own call.
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
/** @taint-sink html $out */
function renderHtml(string $out): void {
}

/** @taint-sink sql $query */
function runQuery(string $query): void {
}

/** @taint-sink shell $cmd */
function runShell(string $cmd): void {
}

renderHtml((string) $_GET["a"]);
runQuery((string) $_GET["b"]);
runShell((string) $_GET["c"]);
===expect===
TaintedHtml@14:0-14:31: Tainted HTML output — possible XSS
TaintedSql@15:0-15:29: Tainted SQL query — possible SQL injection
TaintedShell@16:0-16:29: Tainted shell command — possible command injection
