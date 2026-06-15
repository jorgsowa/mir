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
  takes_int("string");
}

function takes_int(int $i) {}
===expect===
RedundantCondition@7:6-7:29: Condition is always true/false for type 'bool'
InvalidArgument@8:12-8:20: Argument $i of takes_int() expects 'int', got '"string"'
