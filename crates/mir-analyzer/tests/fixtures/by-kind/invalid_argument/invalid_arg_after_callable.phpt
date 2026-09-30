===description===
Invalid arg after callable
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
/**
 * @param callable $callback
 * @return void
 */
function route($callback) {
  if (!is_callable($callback)) {  }
//    ^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
  takes_int("string");
//          ^^^^^^^^ InvalidArgument: Argument $i of takes_int() expects 'int', got '"string"'
}

function takes_int(int $i) {}
===expect===
