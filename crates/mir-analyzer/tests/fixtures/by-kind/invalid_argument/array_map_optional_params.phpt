===description===
array_map with optional parameters
===config===
suppress=UnusedVariable
===file===
<?php

function foo(int $a, string $b = "default") : string {
  return "{$a}:{$b}";
}

// 2 arrays, foo requires 1 parameter and accepts 2 (one optional)
// Should pass - foo can accept 1 or 2 arguments
$result = array_map("foo", [1, 2], [3, 4]);

===expect===
