===description===
Static calls (`Cls::m()`, `self::m()`, `static::m()`, `parent::m()`) define a
by-ref out variable in the true branch of `&&`.
===file===
<?php
class Base {
    public static function parse(string $in, ?int &$out): bool {
        $out = (int) $in;
        return true;
    }
}

final class Child extends Base {
    public static function viaClass(string $in, bool $b): int {
        if ($b && Base::parse($in, $out)) {
            return $out ?? 0;
        }
        return 0;
    }

    public static function viaSelf(string $in, bool $b): int {
        if ($b && self::parse($in, $out)) {
            return $out ?? 0;
        }
        return 0;
    }

    public static function viaStatic(string $in, bool $b): int {
        if ($b && static::parse($in, $out)) {
            return $out ?? 0;
        }
        return 0;
    }

    public static function viaParent(string $in, bool $b): int {
        if ($b && parent::parse($in, $out)) {
            return $out ?? 0;
        }
        return 0;
    }
}
