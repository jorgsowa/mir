===description===
`$list + [1 => null]` can hold the right operand's `null`
===file===
<?php
function f(string $s): ?string {
    [$user, $pass] = explode(':', $s, 2) + [1 => null];
    return null !== $pass ? $pass : $user;
}
