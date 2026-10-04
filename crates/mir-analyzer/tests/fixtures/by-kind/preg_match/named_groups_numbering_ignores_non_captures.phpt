===description===
Escaped parens, classes, lookarounds, non-capturing groups and verbs do not consume group numbers.
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
    preg_match('~(*UTF8)\((?:a)[(]([[:alpha:](]+)(?<=b)(?<n>c)~u', $s, $m);
    /** @mir-check $m is array{0: string, 1: string, 'n': string, 2: string} */
    $_ = $m;
}

===expect===
