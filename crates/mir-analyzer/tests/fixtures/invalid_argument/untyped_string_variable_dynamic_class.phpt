===description===
Untyped string variable dynamic class
===file===
<?php
class ValidClass {
    public function method() {
        return "result";
    }
}

// Without class-string type hint, dynamic class instantiation is invalid
function createInstance($classNameString) {
    /** @mir-check $classNameString is mixed */
    return new $classNameString();
}

$name = "ValidClass";
$obj = createInstance($name);
$obj->method();
===expect===
InvalidStringClass@11:16: Dynamic class instantiation requires string or class-string type, got 'mixed'
