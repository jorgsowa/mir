===description===
A truthy `$x?->m()` proves `$x` non-null, so `if (!$x?->m()) { throw }` leaves it non-null.
===file===
<?php
final class W { public function ok(): bool { return true; } }
function maybe(): ?W { return rand(0, 1) ? new W() : null; }

function f(): W {
    $w = maybe();
    if (!$w?->ok()) {
        throw new \LogicException();
    }
    return $w;
}

function g(): W {
    $w = maybe();
    if ($w?->ok()) {
        return $w;
    }
    return $w;
//  ^^^^^^^^^^ NullableReturnStatement: Return type 'W|null' is not compatible with declared 'W'
}
