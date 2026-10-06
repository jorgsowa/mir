===description===
Bad array map array callable
===file===
<?php
class one { public function two(string $_p): void {} }
//                              ^^^^^^^^^^ UnusedParam: Parameter $_p is never used
array_map(["two", "three"], ["one", "two"]);
//        ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $callback of array_map() expects 'callable', got 'array{0: "two", 1: "three"}'
