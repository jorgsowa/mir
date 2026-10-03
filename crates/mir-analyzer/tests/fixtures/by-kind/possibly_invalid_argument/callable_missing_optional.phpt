===description===
Callable missing optional
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable(string=):bool $arg
 * @return void
 */
function foo($arg) {}

function bar(): bool {
    return rand(0, 10) > 5 ? true : false;
}

foo("bar");
===expect===
