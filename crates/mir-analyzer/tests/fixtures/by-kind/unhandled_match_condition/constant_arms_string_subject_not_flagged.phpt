===description===
A plain string/int subject matched only against compile-time constant arms (typed class constants, enum case ->value/->name, constant array elements) is a lookup, not reported; a runtime arm or unknown constant still is.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Named {
    /** @var string */
    const KEY = 'key';
}

enum Level: string {
    case View = 'view';
    case Edit = 'edit';
}

enum Plain {
    case One;
    case Two;
}

class Registry implements Named {
    const SLOT = ['id' => 'slot-id', 'name' => 'slot-name'];
    const NESTED = ['a' => ['b' => 'deep']];
    const LIST = ['x', 'y'];
    const DYNAMIC = PHP_INT_SIZE . 'x';

    public static function typedConstant(string $k): int {
        $r = match ($k) {
            Named::KEY => 1,
            self::KEY => 2,
        };
        /** @mir-check $r is int */
        return $r;
    }

    public static function enumValue(string $k): int {
        $r = match ($k) {
            Level::View->value => 1,
            Level::Edit->value => 2,
        };
        /** @mir-check $r is int */
        return $r;
    }

    public static function enumName(string $k): int {
        $r = match ($k) {
            Plain::One->name => 1,
            Plain::Two->name => 2,
        };
        /** @mir-check $r is int */
        return $r;
    }

    public static function arrayElement(string $k): int {
        $r = match ($k) {
            self::SLOT['id'] => 1,
            static::SLOT['name'] => 2,
            self::NESTED['a']['b'] => 3,
            self::LIST[0] => 4,
        };
        /** @mir-check $r is int */
        return $r;
    }

    public static function mixedWithLiteral(string $k): int {
        return match ($k) {
            'plain' => 1,
            Level::View->value => 2,
            self::SLOT['id'] => 3,
        };
    }

    public static function nonLiteralConstant(string $k): int {
        return match ($k) {
            self::DYNAMIC => 1,
        };
    }

    public static function runtimeProperty(string $k, Level $level): int {
        return match ($k) {
            Level::View->value => 1,
            $level->value => 2,
        };
    }

    public static function runtimeIndex(string $k, string $i): int {
        return match ($k) {
            self::SLOT[$i] => 1,
        };
    }

    public static function runtimeBase(string $k, array $arr): int {
        return match ($k) {
            $arr['id'] => 1,
        };
    }

    public static function unknownConstant(string $k): int {
        return match ($k) {
            self::MISSING => 1,
        };
    }
}
===expect===
UnhandledMatchCondition@76:15-79:9: Unhandled match condition: possibly-unmatched value of type 'string'
UnhandledMatchCondition@83:15-85:9: Unhandled match condition: possibly-unmatched value of type 'string'
UnhandledMatchCondition@89:15-91:9: Unhandled match condition: possibly-unmatched value of type 'string'
UnhandledMatchCondition@95:15-97:9: Unhandled match condition: possibly-unmatched value of type 'string'
UndefinedConstant@96:12-96:25: Constant Registry::MISSING is not defined
