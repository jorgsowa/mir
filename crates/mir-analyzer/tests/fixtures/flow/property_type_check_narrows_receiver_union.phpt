===description===
A type check on a property drops the receiver classes whose declared property
type can never pass it, in both branches.
===file===
<?php
final class Str { public string $v = ''; }
final class Num { public int $v = 0; }
final class Any { public mixed $v = null; }

function pick(Str|Num $b): Str {
    if (is_string($b->v)) {
        return $b;
    }
    /** @mir-check $b is Num */
    echo 1;
    throw new \LogicException();
}

function pickNegated(Str|Num $b): Num {
    if (!is_string($b->v)) {
        return $b;
    }
    throw new \LogicException();
}

function keepsUndecidedClass(Str|Num|Any $b): void {
    if (is_int($b->v)) {
        /** @mir-check $b is Num|Any */
        echo 1;
    }
}

function nullableReceiver(Str|Num|null $b): void {
    if (is_int($b?->v)) {
        /** @mir-check $b is Num */
        echo 1;
    }
}

function pickEither(Str|Num $b): Str {
    if ($b->v instanceof \Stringable || is_string($b->v)) {
        return $b;
    }
    throw new \LogicException();
}

function pickEitherTypeFn(Str|Num|Any $b): void {
    if (is_float($b->v) || is_string($b->v)) {
        /** @mir-check $b is Any|Str */
        echo 1;
    }
}
