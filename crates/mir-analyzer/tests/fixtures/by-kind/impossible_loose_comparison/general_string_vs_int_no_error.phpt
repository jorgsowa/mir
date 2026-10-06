===description===
Conservative: a general string type (not a literal) vs int should not be flagged.
The string could be "0", "123", or any numeric value that would equal an integer.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
function test(string $s, int $n): void {
    if ($s == $n) {}
}
