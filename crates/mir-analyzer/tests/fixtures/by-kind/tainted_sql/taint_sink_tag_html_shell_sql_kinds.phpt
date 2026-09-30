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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
runQuery((string) $_GET["b"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
runShell((string) $_GET["c"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
===expect===
