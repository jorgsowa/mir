===description===
Callable missing optional multiple params
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable(string, string, string, string=):bool $arg
 * @return void
 */
function foo($arg) {}

function bar(string $a, string $b, string $c): bool {}
//                                             ^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'bool'

foo("bar");
===expect===
