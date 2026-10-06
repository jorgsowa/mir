===description===
SKIPPED-preventUnpackingPossiblyArray
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(int $arg1, int $arg2): void {}

/** @var array<int, int>|object */
$test = [1, 2];
foo(...$test);
