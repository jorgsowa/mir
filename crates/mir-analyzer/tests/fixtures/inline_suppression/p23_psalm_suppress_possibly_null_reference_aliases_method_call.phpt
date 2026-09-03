===description===
P23: `@psalm-suppress PossiblyNullReference` (Psalm's name for member access on a
possibly-null receiver) must suppress mir's own `PossiblyNullMethodCall` kind — before
the fix, `KindSet::matches` compared only against `IssueKind::name()`/`code()` with no
alias table, so the underlying issue still fired AND the suppression itself was flagged
`UnusedSuppress` (double noise instead of silence). Found in egulias-email-validator.
===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    /** @psalm-suppress PossiblyNullReference */
    $obj->bar();
}
===expect===
