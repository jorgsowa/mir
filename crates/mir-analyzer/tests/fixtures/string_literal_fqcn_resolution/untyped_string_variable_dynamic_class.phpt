===description===
untypedStringVariableDynamicClass
===file===
<?php
class ValidClass {
    public function method() {
        return "result";
    }
}

// Without class-string type hint, dynamic class instantiation is invalid
function createInstance($classNameString) {
    return new $classNameString();
}

$name = "ValidClass";
$obj = createInstance($name);
$obj->method();
===expect===
UndefinedClass@10:15: Class <dynamic> does not exist
