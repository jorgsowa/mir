===description===
Bad array map array callable
===file===
<?php
class one { public function two(string $_p): void {} }
array_map(["two", "three"], ["one", "two"]);
===expect===
UnusedParam@2:33-2:43: Parameter $_p is never used
InvalidArgument@3:11-3:27: Argument $callback of array_map() expects 'callable', got 'array{0: "two", 1: "three"}'
