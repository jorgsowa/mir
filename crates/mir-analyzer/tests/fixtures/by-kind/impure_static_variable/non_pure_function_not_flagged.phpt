===description===
ImpureStaticVariable does NOT fire inside a function that is NOT marked @pure.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function impure(): int {
    static $count = 0;
    return ++$count;
}
