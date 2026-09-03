===description===
A multi-line `/** ... @phpstan-ignore ... */` docblock directly above a
single-line statement previously suppressed nothing: the scan that hunts for
the directive's target line landed on the docblock's own closing `*/`
instead of walking past it to the statement below.
===file===
<?php
function test(): void {
    /**
     * @phpstan-ignore undefinedClass
     */
    new NoSuchClass();
}
===expect===
