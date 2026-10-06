===description===
Generic array reads retain their value type when passed as arguments.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<int, string> $map */
function test(array $map): string {
    return strtoupper($map[5]);
}
