===description===
Named and unnamed groups are numbered in order; named groups appear under both keys.
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
    preg_match("/(\\d)(?<a>x)(?P<b>y)(?'c'z)/", $s, $m);
    /** @mir-check $m is array{0: string, 1: string, 'a': string, 2: string, 'b': string, 3: string, 'c': string, 4: string} */
    $_ = $m;
}
