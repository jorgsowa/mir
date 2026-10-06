===description===
Array filter too few args
===file===
<?php
function foo(int $i, string $s) : bool {
//           ^^^^^^ UnusedParam: Parameter $i is never used
//                   ^^^^^^^^^ UnusedParam: Parameter $s is never used
  return true;
}

array_filter([1, 2, 3], "foo");
//                      ^^^^^ InvalidArgument: Argument $callback of array_filter() expects 'callable accepting 1 argument', got 'callable accepting 2 arguments'
