===description===
Invalid arg after callable
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable $callback
 * @return void
 */
function route($callback) {
  if (!is_callable($callback)) {  }
//    ^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
  takes_int("string");
//          ^^^^^^^^ InvalidArgument: Argument $i of takes_int() expects 'int', got '"string"'
}

function takes_int(int $i) {}
===expect===
