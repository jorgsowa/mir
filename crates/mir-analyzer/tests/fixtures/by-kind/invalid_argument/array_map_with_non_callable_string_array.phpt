===description===
Array map with non callable string array
===file===
<?php
$foo = ["one", "two"];
array_map($foo, ["hello"]);
//        ^^^^ InvalidArgument: Argument $callback of array_map() expects 'callable', got 'array{0: "one", 1: "two"}'
