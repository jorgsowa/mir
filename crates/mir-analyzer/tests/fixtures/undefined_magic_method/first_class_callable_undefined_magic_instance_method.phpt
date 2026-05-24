===description===
FirstClassCallable:UndefinedMagicInstanceMethod
===file===
<?php
class Test {
    public function __call(string $name, array $args): mixed {
        return match ($name) {
            default => throw new Error("Undefined method"),
        };
    }
}
$test = new Test();
$closure = $test->length(...);
$length = $closure();

===expect===
UndefinedMagicMethod
===ignore===
TODO
