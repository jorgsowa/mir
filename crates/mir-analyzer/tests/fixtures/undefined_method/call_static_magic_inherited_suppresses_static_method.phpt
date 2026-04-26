===file===
<?php
class Base {
    public static function __callStatic(string $name, array $arguments): mixed {
        return null;
    }
}
class Child extends Base {}
function test(): void {
    Child::anything();
    Child::anotherMissing(1, 2);
}
===expect===
