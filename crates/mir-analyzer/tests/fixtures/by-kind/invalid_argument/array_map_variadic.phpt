===description===
array_map with variadic callback
===config===
suppress=UnusedVariable
===file===
<?php

function foo(int $a, string ...$rest) : string {
  return "{$a}:" . implode(",", $rest);
}

// 3 arrays, foo accepts 1+ arguments with variadic
// Should pass - variadic accepts any number of arguments
$result = array_map("foo", [1, 2], [3, 4], [5, 6]);

===expect===
