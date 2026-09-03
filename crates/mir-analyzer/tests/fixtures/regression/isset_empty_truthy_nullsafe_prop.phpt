===description===
`isset($obj?->prop)`, `!empty($obj?->prop)`, and bare `if ($obj?->prop)`
must narrow the receiver itself to non-null (not just the property) — a
null receiver's `->`/`?->` access is itself unset/falsy, so proving the
condition true also proves the receiver wasn't null. Also fixes the
extractor gap: all three arms only matched plain `->`, missing `?->`.
===config===
suppress=UnusedVariable,MissingConstructor,PossiblyNullPropertyFetch
===file===
<?php
final class Box {
    public ?string $value = null;
}

function issetNullsafe(?Box $box): void {
    if (isset($box?->value)) {
        /** @mir-check $box->value is string */
        $_ = 1;
    }
}

function bareTruthyNullsafe(?Box $box): void {
    if ($box?->value) {
        /** @mir-check $box->value is non-empty-string */
        $_ = 1;
    }
}

function emptyNullsafe(?Box $box): void {
    if (!empty($box?->value)) {
        /** @mir-check $box->value is non-empty-string */
        $_ = 1;
    }
}

function bareFalsyLeavesNullable(?Box $box): void {
    if (!$box?->value) {
        /** @mir-check $box->value is ?string */
        $_ = 1;
    }
}
===expect===
