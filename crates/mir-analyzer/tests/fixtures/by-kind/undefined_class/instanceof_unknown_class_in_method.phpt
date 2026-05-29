===description===
instanceof unknown class in method
===file===
<?php
class A {
    public function f(mixed $x): bool {
        return $x instanceof UnknownClass;
    }
}
===expect===
UndefinedClass@4:30-4:42: Class UnknownClass does not exist
