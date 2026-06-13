===description===
class-string<T> in return type is substituted with the inferred class
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
class Bar {}

/**
 * @template T of object
 * @param class-string<T> $cls
 * @return class-string<T>
 */
function identity(string $cls): string { return $cls; }

$foo = identity(Foo::class);
$bar = identity(Bar::class);
/** @mir-check $foo is class-string<Foo> */
/** @mir-check $bar is class-string<Bar> */
===expect===
