===description===
psalmPropertyReadInvalidFetch
===file===
<?php
/**
 * @psalm-property-read string $foo
 */
class A {
    /** @return mixed */
    public function __get(string $name) {
        if ($name === "foo") {
            return "hello";
        }
    }
}

$a = new A();
echo count($a->foo);
===expect===
InvalidArgument@15:11: Argument $value of count() expects 'array<mixed, mixed>|Countable', got 'string'
