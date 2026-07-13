===description===
A multi-line @property-read array shape is still parsed and checked, not
silently dropped — same fix as the multi-line @param shape.
===file===
<?php
/**
 * @property-read array{
 *     x: int,
 * } $foo
 */
class A {
    /** @return mixed */
    public function __get(string $name) {
        if ($name === "foo") {
            return ["x" => 1];
        }
    }
}

$a = new A();
echo strlen($a->foo);
===expect===
InvalidArgument@17:12-17:19: Argument $string of strlen() expects 'string', got 'array{'x': int}'
