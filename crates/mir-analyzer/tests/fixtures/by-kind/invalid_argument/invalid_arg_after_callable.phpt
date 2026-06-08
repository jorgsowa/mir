===description===
Invalid arg after callable
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
RedundantCondition@7:7-7:30: Condition is always true/false for type 'bool'
InvalidArgument@8:13-8:21: Argument $i of takes_int() expects 'int', got '"string"'
