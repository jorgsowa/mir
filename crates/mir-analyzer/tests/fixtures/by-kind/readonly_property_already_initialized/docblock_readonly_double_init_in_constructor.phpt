===description===
A `@readonly`-only (non-native) property assigned twice in its own
constructor is flagged the same way a native `readonly` property is — the
docblock tag makes the same "write once" contract, just unenforced by PHP
itself.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
class Point {
    /** @readonly */
    public $x;

    public function __construct(int $x) {
        $this->x = $x;
        $this->x = $x + 1;
    }
}
===expect===
ReadonlyPropertyAlreadyInitialized@8:8-8:25: Cannot modify readonly property Point::$x — already initialized
