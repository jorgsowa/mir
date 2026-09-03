===description===
`$x?->prop` (nullsafe) must narrow the same way `$x->prop` already does
across the literal-comparison arms — bool/string/int/enum-case
comparisons only tried the plain `->` extractor, so a nullsafe receiver
silently skipped narrowing entirely, unlike the null/instanceof arms
which already handle both operator forms.
===config===
suppress=UnusedVariable
===file===
<?php
enum Status {
    case Active;
    case Done;
}

class Box {
    public ?string $tag = null;
    public ?bool $flag = null;
    public ?int $count = null;
    public ?Status $state = null;
}

function stringLiteral(?Box $x): void {
    if ($x?->tag === 'active') {
        /** @mir-check $x->tag is 'active' */
        $_ = 1;
    }
}

function intLiteral(?Box $x): void {
    if ($x?->count === 5) {
        /** @mir-check $x->count is 5 */
        $_ = 1;
    }
}

function boolLiteral(?Box $x): void {
    if ($x?->flag === true) {
        /** @mir-check $x->flag is true */
        $_ = 1;
    }
}

function enumCase(?Box $x): void {
    if ($x?->state === Status::Active) {
        /** @mir-check $x->state is Status::Active */
        $_ = 1;
    }
}
===expect===
