===description===
sprintf() returns non-empty-string when the format string guarantees it
(literal prefix/suffix, %d/%f specifiers, %%).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param positive-int $n */
function test_literal_prefix(int $n): void {
    $r = sprintf("id=%d", $n);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_percent_d(): void {
    $r = sprintf("%d", 42);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_literal_suffix(string $s): void {
    $r = sprintf("%s_suffix", $s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_escaped_percent(): void {
    $r = sprintf("100%%");
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_float_format(): void {
    $r = sprintf("%.2f", 3.14);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_pure_string_format_is_not_narrowed(string $s): void {
    $r = sprintf("%s", $s);
    /** @mir-check $r is string */
    $_ = $r;
}
