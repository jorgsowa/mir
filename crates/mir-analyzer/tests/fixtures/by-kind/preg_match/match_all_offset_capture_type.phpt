===description===
preg_match_all with PREG_OFFSET_CAPTURE writes list<list<array{0: string, 1: int}>>
to $matches. Each leaf is [matched_text, byte_offset].
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
    preg_match_all('/(\d+)/', $s, $matches, PREG_OFFSET_CAPTURE);
    /** @mir-check $matches is list<list<array{0: string, 1: int}>> */
    $_ = $matches;
}
===expect===
