===description===
`($obj->prop ?? FALLBACK) !== FALLBACK` (and its `?->` variant) narrows the
receiver itself to non-null, not just the property — a nullable receiver's
`->`/`?->` access coalesces straight to FALLBACK, so a non-FALLBACK result
also proves the receiver wasn't null. Without this, a later plain read of
the same property re-admits null via the receiver's own nullability. Also
covers the `?->` extractor gap: the strict arm only matched plain `->`.
===config===
suppress=UnusedVariable,MissingConstructor,PossiblyNullPropertyFetch
===file===
<?php
final class Box {
    public ?string $value = null;
}

function narrowsOnFalseBranchNullsafe(?Box $box): void {
    if (($box?->value ?? 'default') !== 'default') {
        /** @mir-check $box->value is string */
        $_ = 1;
    }
}

function reversedOperandsNullsafe(?Box $box): void {
    if ('default' !== ($box?->value ?? 'default')) {
        /** @mir-check $box->value is string */
        $_ = 1;
    }
}

function trueBranchLeavesNullableNullsafe(?Box $box): void {
    if (($box?->value ?? 'default') === 'default') {
        /** @mir-check $box->value is ?string */
        $_ = 1;
    }
}

function narrowsReceiverOnPlainArrow(?Box $box): void {
    if (($box->value ?? 'default') !== 'default') {
        /** @mir-check $box->value is string */
        $_ = 1;
    }
}

function narrowsReceiverOnLooseComparison(?Box $box): void {
    if (($box->value ?? 'default') != 'default') {
        /** @mir-check $box->value is string */
        $_ = 1;
    }
}
===expect===
