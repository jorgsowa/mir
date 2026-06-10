===description===
Construct callable from class string array
===file===
<?php
interface Foo {
    public function bar() : int;
}

/**
 * @param callable():string $c
 */
function takesCallableReturningString(callable $c) : void {
    $c();
}

/**
 * @param class-string<Foo> $c
 */
function foo(string $c) : void {
    takesCallableReturningString([$c, "bar"]);
}
===expect===
