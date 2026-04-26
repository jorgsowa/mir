===file===
<?php
class Magic {
    public static function __callStatic(string $name, array $arguments): mixed {
        return null;
    }
}
function test(): void {
    Magic::anything();
    Magic::anotherMissing(1, 2);
}
===expect===
