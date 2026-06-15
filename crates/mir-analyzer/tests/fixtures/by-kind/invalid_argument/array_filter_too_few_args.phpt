===description===
Array filter too few args
===file===
<?php
function foo(int $i, string $s) : bool {
  return true;
}

array_filter([1, 2, 3], "foo");
===expect===
UnusedParam@2:13-2:19: Parameter $i is never used
UnusedParam@2:21-2:30: Parameter $s is never used
InvalidArgument@6:24-6:29: Argument $callback of array_filter() expects 'callable accepting 1 argument', got 'callable accepting 2 arguments'
