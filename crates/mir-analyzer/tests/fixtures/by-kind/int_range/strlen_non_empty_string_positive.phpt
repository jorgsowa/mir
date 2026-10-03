===description===
strlen() on non-empty-string returns int<1, max>; on plain string returns int<0, max>
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test_string(string $s): void {
    $n = strlen($s);
    /** @mir-check $n is int<0, max> */
    $_ = $n;
}

/** @param non-empty-string $s */
function test_non_empty(string $s): void {
    $n = strlen($s);
    /** @mir-check $n is int<1, max> */
    $_ = $n;
}
===expect===
