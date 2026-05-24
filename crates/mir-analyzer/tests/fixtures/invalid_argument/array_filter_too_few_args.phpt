===description===
arrayFilterTooFewArgs
===file===
<?php
function foo(int $i, string $s) : bool {
  return true;
}

array_filter([1, 2, 3], "foo");
===expect===
UnusedParam@2:14: Parameter $i is never used
UnusedParam@2:22: Parameter $s is never used
InvalidArgument@6:25: Argument $callback of array_filter() expects 'callable accepting 1 arg(s)', got 'callable accepting 2 argument(s)'
