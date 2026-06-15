===description===
Array map too many args
===file===
<?php
function foo() : bool {
  return true;
}

array_map("foo", [1, 2, 3]);
===expect===
TooManyArguments@6:10-6:15: Too many arguments for foo(): expected 0, got 1
