===description===
preg_match without PREG_OFFSET_CAPTURE writes list<string> to $matches.
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
    preg_match('/(\d+)/', $s, $matches);
    /** @mir-check $matches is list<string> */
    $_ = $matches;
}
