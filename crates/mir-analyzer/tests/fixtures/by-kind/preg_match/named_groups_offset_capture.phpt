===description===
Named groups with PREG_OFFSET_CAPTURE keep the per-entry offset shape.
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
    preg_match('/(?<a>x)/', $s, $m, PREG_OFFSET_CAPTURE);
    /** @mir-check $m is array{0: array{0: string, 1: int}, 'a': array{0: string, 1: int}, 1: array{0: string, 1: int}} */
    $_ = $m;
}
