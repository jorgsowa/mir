===description===
`empty($mixed)` leaves the falsy string `"0"` possible.
===file===
<?php
function f(mixed $value): bool {
    return empty($value) && $value !== '0';
}
