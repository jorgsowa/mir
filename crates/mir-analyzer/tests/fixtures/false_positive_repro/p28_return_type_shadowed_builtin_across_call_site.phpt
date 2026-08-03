===description===
P28 KNOWN RESIDUAL GAP (not yet fixed): P28's fix reconciles a docblock-
shadowed builtin return type at body-analysis flow-seeding time (within the
declaring function/method's own body), and centrally in
`find_property_in_chain` for properties — but a *caller* elsewhere invoking
a method and chaining a call onto its return value still reads the
unreconciled stored type via a separate call-resolution path
(`call/method.rs`), not body-analysis seeding.
mir currently emits (the bug): UndefinedMethod@21:11-21:30 (Generator::build)
plus a collateral MixedReturnStatement@21:4-21:31 (the call resolves to
`mixed`, not a declared return type mismatch).
Expected: no issue. Remove the ignore marker below to activate once fixed.
===ignore===
===config===
suppress=UnusedParam
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Factory
{
    /** @return Generator */
    public function make(): Generator
    {
        return new Generator();
    }
}

function useFactory(Factory $f): string
{
    return $f->make()->build();
}
===expect===
