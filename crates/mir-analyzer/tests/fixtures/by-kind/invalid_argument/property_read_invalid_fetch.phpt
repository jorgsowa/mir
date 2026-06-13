===description===
Property read invalid fetch
===file===
<?php
/**
 * @property-read string $foo
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
MissingConstructor@5:0-5:9: Class A has uninitialized properties but no constructor
InvalidArgument@15:12-15:19: Argument $value of count() expects 'array<mixed, mixed>|Countable', got 'string'
