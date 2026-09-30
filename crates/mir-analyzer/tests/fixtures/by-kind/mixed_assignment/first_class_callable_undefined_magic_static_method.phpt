===description===
FirstClassCallable:UndefinedMagicStaticMethod
===config===
suppress=UnusedVariable
===file===
<?php
class Test {
    public static function __callStatic(string $name, array $args): mixed {
        return match ($name) {
            default => throw new Error("Undefined method"),
        };
    }
}
$closure = Test::length(...);
$length = $closure();
//<^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $length is assigned a mixed type

===expect===
