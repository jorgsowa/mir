===description===
Array map too few args
===file===
<?php
function foo(int $i, string $s) : bool {
//           ^^^^^^ UnusedParam: Parameter $i is never used
//                   ^^^^^^^^^ UnusedParam: Parameter $s is never used
  return true;
}

array_map("foo", [1, 2, 3]);
//        ^^^^^ TooFewArguments: Too few arguments for foo(): expected 2, got 1
