===description===
invalidArgAfterCallable
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
InvalidScalarArgument
===ignore===
TODO
