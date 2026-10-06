===description===
preg_match_all with PREG_SET_ORDER writes list<list<string>> to $matches.
PREG_SET_ORDER changes the ordering (outer = per-match sets) but not element types.
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
    preg_match_all('/(\d+)/', $s, $matches, PREG_SET_ORDER);
    /** @mir-check $matches is list<list<string>> */
    $_ = $matches;
}
