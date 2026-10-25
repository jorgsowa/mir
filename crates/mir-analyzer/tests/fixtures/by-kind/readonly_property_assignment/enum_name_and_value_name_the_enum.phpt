===description===
Writing or unsetting an enum's `name` / `value` names the enum, as PHP does, not the stub interface that declares the property.
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
}

function rename(Suit $s): void {
    $s->name = 'x';
//  ^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Suit::$name outside of constructor
    $s->value = 'y';
//  ^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Suit::$value outside of constructor
    unset($s->value);
//        ^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Suit::$value outside of constructor
}
