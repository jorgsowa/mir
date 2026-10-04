===description===
Patterns that cannot be numbered reliably (x flag, branch reset, duplicate names, dynamic) keep list<string>.
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

function run(string $s, string $dyn): void {
    preg_match('/(?<a>x) # (\n/x', $s, $m1);
    /** @mir-check $m1 is list<string> */
    $_1 = $m1;
    preg_match('/(?|(?<a>x)|(y))/', $s, $m2);
    /** @mir-check $m2 is list<string> */
    $_2 = $m2;
    preg_match('/(?<a>x)|(?<a>y)/J', $s, $m3);
    /** @mir-check $m3 is list<string> */
    $_3 = $m3;
    preg_match($dyn, $s, $m4);
    /** @mir-check $m4 is list<string> */
    $_4 = $m4;
}

===expect===
