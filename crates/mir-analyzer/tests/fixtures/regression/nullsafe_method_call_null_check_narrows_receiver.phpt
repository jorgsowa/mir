===description===
`$bar?->getVal() !== null` must narrow `$bar` itself to non-null when
`getVal()`'s declared return type excludes null — the null-comparison
dispatcher only ever recognized a nullsafe PROPERTY access
(`extract_nullsafe_prop_access`), never a nullsafe METHOD call, so `$bar`
stayed nullable afterwards. A return type that admits null (`maybeVal`)
must not narrow, since the null could then come from the method's own
result instead of the receiver.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Bar {
    public function getVal(): string {
        return 'x';
    }

    public function maybeVal(): ?string {
        return null;
    }

    public function ping(): void {}
}

function narrowsOnNotNull(?Bar $bar): void {
    if ($bar?->getVal() !== null) {
        $bar->ping();
    }
}

function narrowsOnNull(?Bar $bar): void {
    if ($bar?->getVal() === null) {
        /** @mir-check $bar is null */
        $_ = 1;
    }
}

function doesNotNarrowWhenReturnIsNullable(?Bar $bar): void {
    if ($bar?->maybeVal() !== null) {
        $bar->ping();
    }
}
===expect===
PossiblyNullMethodCall@29:8-29:20: Cannot call method ping() on possibly null value
