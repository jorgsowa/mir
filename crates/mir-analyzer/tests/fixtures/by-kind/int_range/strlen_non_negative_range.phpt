===description===
strlen() / mb_strlen() are int<0, max>
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $s): void {
    $n = strlen($s);
    /** @mir-check $n is int<0, max> */
    $_ = $n;
}
===expect===
