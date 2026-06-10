===description===
FirstClassCallable:UndefinedMagicStaticMethod
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

===expect===
MixedAssignment@10:1-10:21: Variable $length is assigned a mixed type
