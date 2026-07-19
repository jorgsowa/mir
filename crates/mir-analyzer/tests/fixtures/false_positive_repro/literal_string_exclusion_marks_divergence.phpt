===description===
`$s === 'c'` against a closed `"a"|"b"` union must mark the branch as
provably unreachable (via mark_diverges), the same way the literal-int
narrowers already do — the literal-string narrowers hardcoded
mark_diverges=false where their int siblings correctly call
contradiction::is_closed_precise (which already lists TLiteralString as
a supported "precise" atom). Covers var, property, and static-property
receivers. The separate DocblockTypeContradiction/ImpossibleIdentical-
Comparison diagnostics fire regardless (an unrelated, always-on static
check) — the `@mir-check $_ is never` reachability probe is what
isolates this specific fix (see the `RedundantCondition` presence).
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
/** @param "a"|"b" $s */
function varUnionMismatchUnreachable(string $s): void {
    if ($s === 'c') {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

/** @param "a"|"b"|string $s */
function varWideStringStillReachable(string $s): void {
    if ($s === 'c') {
        /** @mir-check $s is "c" */
        $_ = 1;
    }
}

class Bag {
    /** @var "a"|"b" */
    public string $label = 'a';

    public function propUnionMismatchUnreachable(): void {
        if ($this->label === 'c') {
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}

class StaticBag {
    /** @var "a"|"b" */
    public static string $label = 'a';

    public static function staticPropUnionMismatchUnreachable(): void {
        if (self::$label === 'c') {
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}
===expect===
DocblockTypeContradiction@4:8-4:18: Type '"a"|"b"' makes '$s === "c"' impossible — this can never hold
ImpossibleIdenticalComparison@4:8-4:18: '===' between '"a"|"b"' and '"c"' is always false — these types can never be identical
RedundantCondition@4:8-4:18: Condition is always true/false for type 'bool'
ImpossibleIdenticalComparison@23:12-23:32: '===' between '"a"|"b"' and '"c"' is always false — these types can never be identical
RedundantCondition@23:12-23:32: Condition is always true/false for type 'bool'
ImpossibleIdenticalComparison@35:12-35:32: '===' between '"a"|"b"' and '"c"' is always false — these types can never be identical
RedundantCondition@35:12-35:32: Condition is always true/false for type 'bool'
