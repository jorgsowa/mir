===description===
Regression guard: a single-line `/** @phpstan-ignore */` docblock above a
statement already worked before the multi-line docblock fix and must keep
working — the docblock's own closing `*/` sits on the same line as the
directive, so there's nothing to walk past.
===file===
<?php
function test(): void {
    /** @phpstan-ignore undefinedClass */
    new NoSuchClass();
}
===expect===
