===description===
When the flags argument is a non-literal (variable or unknown), preg_match falls
back to list<string> conservatively (no PREG_OFFSET_CAPTURE assumed).
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

function run(string $s, int $flags): void {
    preg_match('/(\d+)/', $s, $matches, $flags);
    /** @mir-check $matches is list<string> */
    $_ = $matches;
}
