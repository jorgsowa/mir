===description===
`extract()` and `array_multisort()` take their array by reference only when they can
(prefer-ref), so a call result is accepted; other by-ref params still reject it.
===file===
<?php
function vars(): array { return ['a' => 1]; }
function f(): void {
    \extract(vars());
    \array_multisort(vars());
    \sort(vars());
//        ^^^^^^ InvalidPassByReference: Argument $array of sort() must be passed by reference
}
