===description===
Invalid array callable
===file===
<?php
function foo(callable $callback) : void {
    $callback();
}

final class Bar {
    public static function baz() : void {}
}

foo([Bar::class, "baz", 1231233]);
===expect===
InvalidArgument@10:5-10:33: Argument $callback of callable() expects 'callable (string or [object, "method"])', got 'array{0: class-string<Bar>, 1: "baz", 2: 1231233}'
