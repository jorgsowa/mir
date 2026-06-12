===description===
Untyped string variable dynamic class — an untyped (mixed) param is not
InvalidStringClass; mixed is already imprecise (a Mixed* concern)
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
