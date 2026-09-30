===description===
Readonly redeclaration of an abstract get-only hook property in a parent class is valid
===file===
<?php
abstract class Base {
    abstract public string $name { get; }
}

final class Impl extends Base {
    public function __construct(public readonly string $name) {}
}

function name_of(Impl $i): string {
    $n = $i->name;
    /** @mir-check $n is string */
    return $n;
}
===expect===
