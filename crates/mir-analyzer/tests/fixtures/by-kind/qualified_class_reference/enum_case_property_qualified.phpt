===description===
An enum case literal's `->value` and `->name` resolve through fully qualified and relative names.
===file===
<?php
namespace App;

enum E: int {
    case One = 1;
}

function f(): int {
    $a = \App\E::One->value;
    $b = namespace\E::One->value;
    return $a + $b;
}

function g(): string {
    return \App\E::One->name . namespace\E::One->name;
}
