===description===
Array map with non callable string array
===file===
<?php
$foo = ["one", "two"];
array_map($foo, ["hello"]);
===expect===
InvalidArgument@3:10-3:14: Argument $callback of array_map() expects 'callable', got 'array{0: "one", 1: "two"}'
