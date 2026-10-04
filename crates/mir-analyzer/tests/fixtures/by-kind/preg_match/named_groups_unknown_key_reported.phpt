===description===
Reading a name the pattern does not define is still reported.
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

function run(string $s): string {
    preg_match('/(?<a>x)/', $s, $m);
    return (string) $m['b'];
}

===expect===
NonExistentArrayOffset@5:23-5:26: Array offset 'b' does not exist
