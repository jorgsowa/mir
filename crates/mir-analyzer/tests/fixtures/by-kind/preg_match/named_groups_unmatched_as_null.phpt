===description===
Named groups with PREG_UNMATCHED_AS_NULL admit null in every entry.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function run(string $s): void {
    preg_match('/(?<a>x)?/', $s, $m, PREG_UNMATCHED_AS_NULL);
    /** @mir-check $m is array{0: string|null, 'a': string|null, 1: string|null} */
    $_ = $m;
}
