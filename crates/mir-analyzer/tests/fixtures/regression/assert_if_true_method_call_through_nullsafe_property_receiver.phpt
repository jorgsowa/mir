===description===
`@psalm-assert-if-true int $p` on a method reached through a NULLSAFE
PROPERTY receiver (`$w?->inner->isInt($p)`) never narrowed — this is a
different shape from an already-fixed sibling gap (the whole method CALL
itself being nullsafe, `$v?->isPositive($x)`): here the call is a plain
`MethodCall`, but `method_call_receiver_fqcn`'s object-resolution only
tried `extract_prop_access` (plain `->` only), missing the nullsafe
receiver chain `extract_any_prop_access` already handles elsewhere.
===config===
suppress=UnusedVariable,MissingParamType,PossiblyNullMethodCall
===file===
<?php
class Foo {
    /** @psalm-assert-if-true int $p */
    public function isInt($p): bool {
        return is_int($p);
    }
}

class Wrapper {
    public ?Foo $inner = null;
}

function check(?Wrapper $w, $p): void {
    if ($w?->inner->isInt($p)) {
        /** @mir-check $p is int */
        $_ = 1;
    }
}
===expect===
