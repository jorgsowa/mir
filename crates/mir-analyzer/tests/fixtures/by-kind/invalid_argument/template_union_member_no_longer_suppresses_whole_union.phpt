===description===
`param_contains_template_or_unknown` (crates/mir-analyzer/src/call/args.rs)
used to be a plain `.any()` over a param union's atoms: if ANY alternative
mentioned an unresolved template anywhere in its own type args (here
`Bar<T>` in `Foo|Bar<T>`), argument checking was skipped for the ENTIRE
union, even though a candidate argument satisfies NEITHER alternative.
Fixed by checking each `TNamedObject` atom against the argument's own
class (mirroring how the `TIntersection` arm already forgave only the
templated part while still enforcing the concrete parts) — a bare string
now correctly mismatches both `Foo` and any instantiation of `Bar<T>`.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php
class Foo {}

/** @template T */
class Bar {
    /** @param T $x */
    public function __construct($x) {}
}

/**
 * @template T
 * @param Foo|Bar<T> $x
 */
function takesFooOrBar($x): void {}

takesFooOrBar("plain-string");
===expect===
InvalidArgument@16:14-16:28: Argument $x of takesFooOrBar() expects 'Foo|Bar<T>', got '"plain-string"'
