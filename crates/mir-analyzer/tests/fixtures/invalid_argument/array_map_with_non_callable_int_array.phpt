===description===
arrayMapWithNonCallableIntArray
===file===
<?php
$foo = [1, 2];
array_map($foo, ["hello"]);
===expect===
InvalidArgument@3:11: Argument $callback of array_map() expects 'callable', got 'array{0: 1, 1: 2}'
