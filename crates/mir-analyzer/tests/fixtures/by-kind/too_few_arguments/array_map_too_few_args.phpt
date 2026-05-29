===description===
Array map too few args
===file===
<?php
function foo(int $i, string $s) : bool {
  return true;
}

array_map("foo", [1, 2, 3]);
===expect===
UnusedParam@2:14-2:20: Parameter $i is never used
UnusedParam@2:22-2:31: Parameter $s is never used
TooFewArguments@6:11-6:16: Too few arguments for foo(): expected 2, got 1
