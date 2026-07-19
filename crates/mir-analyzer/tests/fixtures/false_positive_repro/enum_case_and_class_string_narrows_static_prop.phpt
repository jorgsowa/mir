===description===
`self::$prop === EnumName::CaseName` and `self::$prop === Foo::class` (both
operand orders, and `static::$prop`) must narrow the static property —
the top-level dispatch guard treats any StaticPropertyAccess operand as a
candidate enum-case node first; when that fails (a genuine property, not
an enum case), it fell through to nothing instead of treating the
property as a receiver and checking the other side for an enum-case or
class-const target.
===config===
suppress=MissingConstructor
===file===
<?php
enum Status {
    case Active;
    case Inactive;
}
class Foo {}

class Box {
    protected static Status $s = Status::Active;
    protected static string $cls = Foo::class;

    public static function useEnumCase(): void {
        if (self::$s === Status::Active) {
            /** @mir-check self::$s is Status::Active */
            echo "";
        }
    }

    public static function useEnumCaseReversed(): void {
        if (Status::Active === self::$s) {
            /** @mir-check self::$s is Status::Active */
            echo "";
        }
    }

    public static function useClassString(): void {
        if (self::$cls === Foo::class) {
            /** @mir-check self::$cls is class-string<Foo> */
            echo "";
        }
    }

    public static function useClassStringReversed(): void {
        if (Foo::class === self::$cls) {
            /** @mir-check self::$cls is class-string<Foo> */
            echo "";
        }
    }
}

class ChildBox extends Box {
    public static function useViaStatic(): void {
        if (static::$s === Status::Active) {
            /** @mir-check static::$s is Status::Active */
            echo "";
        }
    }
}
===expect===
