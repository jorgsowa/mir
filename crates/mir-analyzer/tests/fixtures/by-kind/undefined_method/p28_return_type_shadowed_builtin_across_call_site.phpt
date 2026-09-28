===description===
P28 sibling: a caller in another function chaining a call onto a method's
docblock-shadowed-builtin return value (`$f->make()->build()`) must resolve
`make()`'s return type the same way body-analysis flow-seeding does, via
the reconciliation in `resolve_method_from_db` (`call/method.rs`).
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
