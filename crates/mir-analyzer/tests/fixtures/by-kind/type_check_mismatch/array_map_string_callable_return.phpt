===description===
array_map with a string callable resolves the named function's return type
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param list<string> $words */
function test(array $words): void {
    $r = array_map('strtoupper', $words);
    /** @mir-check $r is list<string> */
    $_ = $r;
}
