===description===
$obj::class should produce class-string<T> not bare class-string (FP G)
===file===
<?php
/**
 * @param class-string<Throwable> $class
 */
function expectException(string $class): void {
    echo $class;
}

$e = new RuntimeException("oops");
/** @mir-check $e is RuntimeException */
expectException($e::class);

class MyError extends LogicException {}

$err = new MyError();
expectException($err::class);
===expect===
