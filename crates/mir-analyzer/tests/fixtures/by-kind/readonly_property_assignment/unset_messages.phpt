===description===
Unsetting a readonly property reads as PHP's error; the declaring class may unset it to defer initialization.
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
}
final class Lazy {
    public readonly int $value;

    public function __construct() {
//                  ^^^^^^^^^^^ PropertyPossiblyUninitialized: Property Lazy::$value may be left uninitialized by the constructor
        unset($this->value);
    }
}
function reset_value(Lazy $lazy, Suit $suit): void {
    unset($lazy->value);
//        ^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot unset readonly property Lazy::$value
    unset($suit->value);
//        ^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot unset readonly property Suit::$value
}
