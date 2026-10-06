===description===
Psalm property read invalid fetch
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
//         ^^^^^^^ InvalidArgument: Argument $value of count() expects 'array|Countable', got 'string'
