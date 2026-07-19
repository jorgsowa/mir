===description===
`($x ?? FALLBACK) != FALLBACK` (loose comparison) narrows `$x` to non-null,
the same way the strict `!==` form already does — sound because a null $x
always coalesces to exactly FALLBACK, so `FALLBACK == FALLBACK` is
trivially true regardless of loose vs strict.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
function narrowsOnFalseBranchVar(?string $x): void {
    if (($x ?? 'default') != 'default') {
        /** @mir-check $x is string */
        $_ = 1;
    }
}

function reversedOperandsVar(?string $x): void {
    if ('default' != ($x ?? 'default')) {
        /** @mir-check $x is string */
        $_ = 1;
    }
}

function trueBranchLeavesNullableVar(?string $x): void {
    if (($x ?? 'default') == 'default') {
        /** @mir-check $x is ?string */
        $_ = 1;
    }
}

final class Bag {
    /** @var ?string */
    public ?string $label = null;

    public function narrowsOnFalseBranchProp(): void {
        if (($this->label ?? 'default') != 'default') {
            /** @mir-check $this->label is string */
            $_ = 1;
        }
    }
}
===expect===
