===description===
Bad array map array callable
===file===
<?php
class one { public function two(string $_p): void {} }
array_map(["two", "three"], ["one", "two"]);
===expect===
UnusedParam@2:32-2:42: Parameter $_p is never used
InvalidArgument@3:10-3:26: Argument $callback of array_map() expects 'callable', got 'array{0: "two", 1: "three"}'
