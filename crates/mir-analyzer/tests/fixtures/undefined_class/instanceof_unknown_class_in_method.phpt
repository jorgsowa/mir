===source===
<?php
class A {
    public function f(mixed $x): bool {
        return $x instanceof UnknownClass;
    }
}
===expect===
UndefinedClass: UnknownClass
