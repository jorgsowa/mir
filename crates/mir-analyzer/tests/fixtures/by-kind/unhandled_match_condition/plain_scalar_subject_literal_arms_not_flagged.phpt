===description===
A plain int/string subject matched only against literal arms (inline or class constants, even non-literal ones) is not reported; a runtime arm or a large bounded range still is.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Kind {
    const A = 'a';
    const B = 'b';
    const ONE = 1;
    const DYNAMIC = PHP_INT_SIZE . 'x';

    public static function selfConst(string $kind): int {
        return match ($kind) {
            self::A => 1,
            self::B => 2,
        };
    }

    public static function staticConst(string $kind): int {
        return match ($kind) {
            static::A, static::B => 1,
        };
    }

    public static function intConst(int $n): string {
        return match ($n) {
            self::ONE => 'one',
            2 => 'two',
            -1 => 'minus',
        };
    }

    public static function nonLiteralConst(string $kind): int {
        return match ($kind) {
            self::DYNAMIC => 1,
        };
    }

    public static function mixedWithVariable(string $kind, string $other): int {
        return match ($kind) {
//             ^ +3:9 UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'string'
            self::A => 1,
            $other => 2,
        };
    }
}

function otherClassConst(string $kind): int {
    return match ($kind) {
        Kind::A => 1,
        Kind::B => 2,
    };
}

function inlineLiterals(string $kind, int $n): int {
    $a = match ($kind) { 'x' => 1, 'y' => 2 };
    $b = match ($n) { 1 => 1, 2 => 2 };
    return $a + $b;
}

/** @param int<0, 100000> $n */
function largeRangeStillFlagged(int $n): int {
    return match ($n) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'int<0, 100000>'
        0 => 1,
        1 => 2,
    };
}

/** @param 'a'|'b' $kind */
function literalUnionStillFlagged(string $kind): int {
    return match ($kind) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: "b"
        'a' => 1,
    };
}
