===description===
`@mir-check EXPR is TYPE` now parses EXPR as a real PHP expression and runs
it through the same inference every other expression goes through, instead
of only supporting a bare `$variable` name. Property access, array/shape
keys, static property access, chained/nested access, and method-call return
types are all checkable directly, with no intermediate `$x = EXPR;`
assignment needed. Each case below has both a passing (correct) assertion
and a failing (deliberately wrong) one, so a mismatch is proven to still be
caught rather than the check silently always passing.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingPropertyType
===file===
<?php

class Holder {
    public bool|string $flag = true;
}

function checksPropertyDirectly(Holder $h): void {
    if ($h->flag === true) {
        /** @mir-check $h->flag is true */
        $_ = null;
    }
}

function checksPropertyDirectlyMismatch(Holder $h): void {
    if ($h->flag === true) {
        /** @mir-check $h->flag is false */
        $_ = null;
    }
}

/**
 * @param array{name: string, age?: int} $arr
 */
function checksArrayShapeKeyDirectly(array $arr): void {
    if (isset($arr['age'])) {
        /** @mir-check $arr['age'] is int */
        $_ = null;
    }
}

/**
 * @param array{name: string, age?: int} $arr
 */
function checksArrayShapeKeyDirectlyMismatch(array $arr): void {
    if (isset($arr['age'])) {
        /** @mir-check $arr['age'] is string */
        $_ = null;
    }
}

/**
 * @param array{a: array{b: int}} $arr
 */
function checksNestedShapeKeyDirectly(array $arr): void {
    /** @mir-check $arr['a']['b'] is int */
    $_ = null;
}

/**
 * @param array{a: array{b: int}} $arr
 */
function checksNestedShapeKeyDirectlyMismatch(array $arr): void {
    /** @mir-check $arr['a']['b'] is string */
    $_ = null;
}

class Service {
    protected static ?string $name = null;

    public static function checksStaticPropDirectly(): void {
        if (self::$name !== null) {
            /** @mir-check self::$name is string */
            $_ = null;
        }
    }

    public static function checksStaticPropDirectlyMismatch(): void {
        if (self::$name !== null) {
            /** @mir-check self::$name is int */
            $_ = null;
        }
    }

    public function getStatus(): string {
        return 'ok';
    }
}

function checksMethodCallReturnTypeDirectly(Service $s): void {
    /** @mir-check $s->getStatus() is string */
    $_ = null;
}

function checksMethodCallReturnTypeDirectlyMismatch(Service $s): void {
    /** @mir-check $s->getStatus() is int */
    $_ = null;
}

class Inner {
    public bool|string $value = true;
}

class Outer {
    public Inner $inner;
}

// Chained (two-level) property access: read the real, current type, whether
// or not narrowing.rs happens to have narrowed it — a property chain like
// this isn't narrowed today (a separate, pre-existing limitation of
// narrowing.rs's single-hop extract_prop_access, not of @mir-check itself),
// so the honest expectation here is the declared union, unnarrowed.
function checksChainedPropertyDirectly(Outer $o): void {
    /** @mir-check $o->inner->value is bool|string */
    $_ = null;
}

function checksChainedPropertyDirectlyMismatch(Outer $o): void {
    /** @mir-check $o->inner->value is true */
    $_ = null;
}

function checksNullsafeChainDirectly(?Inner $i): void {
    if ($i?->value !== null) {
        /** @mir-check $i?->value is bool|string */
        $_ = null;
    }
}

function checksNullsafeChainDirectlyMismatch(?Inner $i): void {
    if ($i?->value !== null) {
        /** @mir-check $i?->value is int */
        $_ = null;
    }
}

// A malformed EXPR fails to parse and falls back to `mixed` rather than
// panicking or being silently skipped — still surfaces as a mismatch
// (unless TYPE itself is `mixed`), so a typo in the annotation is never
// silently invisible.
function checksMalformedExprFallsBackToMixed(): void {
    /** @mir-check $ is int */
    $_ = null;
}
===expect===
TypeCheckMismatch@17:8-17:18: Type of $h->flag is expected to be false, got true
TypeCheckMismatch@37:8-37:18: Type of $arr['age'] is expected to be string, got int
TypeCheckMismatch@54:4-54:14: Type of $arr['a']['b'] is expected to be string, got int
TypeCheckMismatch@70:12-70:22: Type of self::$name is expected to be int, got string
TypeCheckMismatch@86:4-86:14: Type of $s->getStatus() is expected to be int, got string
TypeCheckMismatch@109:4-109:14: Type of $o->inner->value is expected to be true, got bool|string
TypeCheckMismatch@122:8-122:18: Type of $i?->value is expected to be int, got bool|string
TypeCheckMismatch@132:4-132:14: Type of $ is expected to be int, got mixed
