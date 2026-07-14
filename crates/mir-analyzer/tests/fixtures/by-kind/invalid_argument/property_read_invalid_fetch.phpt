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
InvalidArgument@15:11-15:18: Argument $value of count() expects 'array|Countable', got 'string'
