===description===
A native `array|Traversable` union parameter type — written directly, not
via the `iterable` keyword — must display as the collapsed `iterable`, since
PHP defines `iterable` as exactly `array|Traversable`.
===file===
<?php
class A {}

function f(array|Traversable $x): void { $_ = $x; }

f(new A());
===expect===
InvalidArgument@6:2-6:9: Argument $x of f() expects 'iterable', got 'A'
