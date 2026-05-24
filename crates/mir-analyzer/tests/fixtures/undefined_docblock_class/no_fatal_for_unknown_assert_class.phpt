===description===
noFatalForUnknownAssertClass
===file===
<?php
interface Foo {}

class Bar implements Foo {
    public function sayHello(): void {
        echo "Hello";
    }
}

/**
 * @param mixed $value
 * @param class-string $type
 * @psalm-assert SomeUndefinedClass $value
 */
function assertInstanceOf($value, string $type): void {
    // some code
}

// Returns concreate implementation of Foo, which in this case is Bar
function getImplementationOfFoo(): Foo {
    return new Bar();
}

$bar = getImplementationOfFoo();
assertInstanceOf($bar, Bar::class);

$bar->sayHello();
===expect===
UndefinedDocblockClass
===ignore===
TODO
