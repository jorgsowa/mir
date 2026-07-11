===description===
`$b?->value !== null` narrows both the nullsafe receiver `$b` (a null
receiver would have short-circuited the whole chain to `null`) and the
property `$b->value` itself — matching the plain `$b->value !== null` case.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Box {
    public ?string $value = null;
}

function useValue(string $s): void {}

function test(?Box $b): void {
    if ($b?->value !== null) {
        useValue($b->value);
        /** @mir-check $b is Box */
    }
}
===expect===
