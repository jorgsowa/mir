===description===
MixedArgument does NOT fire when the argument has a concrete (non-mixed) type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(int $a): void {}
foo(42);
