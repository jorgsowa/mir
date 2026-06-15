===description===
Array map too few args
===file===
<?php
function foo(int $i, string $s) : bool {
  return true;
}

array_map("foo", [1, 2, 3]);
===expect===
UnusedParam@2:13-2:19: Parameter $i is never used
UnusedParam@2:21-2:30: Parameter $s is never used
TooFewArguments@6:10-6:15: Too few arguments for foo(): expected 2, got 1
