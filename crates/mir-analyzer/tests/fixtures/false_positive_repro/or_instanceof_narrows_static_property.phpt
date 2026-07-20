===description===
`self::$prop instanceof A || self::$prop instanceof B` must narrow the
static property like its instance-property/plain-variable counterparts
already do, including a mixed instanceof/is_TYPE() disjunct and the
switch(true)/match(true) fallthrough shape.
===config===
suppress=MissingConstructor,UnusedParam,UnusedVariable
===file===
<?php
interface A {}
interface B {}

class Foo {
    private static A|B|int $prop = 0;
    private static A|int|string $prop2 = 0;

    // Positive: pure instanceof OR-disjunct narrows away the `int` alternative.
    public static function test(): void {
        if (self::$prop instanceof A || self::$prop instanceof B) {
            echo get_class(self::$prop);
        }
    }

    // Positive: mixed instanceof/is_TYPE() disjunct narrows away `string`.
    public static function testMixed(): void {
        if (self::$prop2 instanceof A || is_int(self::$prop2)) {
            /** @mir-check self::$prop2 is A|int */
            $_ = self::$prop2;
        }
    }

    // Positive: switch(true) fallthrough on a static property.
    public static function testSwitchTrue(): void {
        switch (true) {
            case self::$prop instanceof A:
            case self::$prop instanceof B:
                echo get_class(self::$prop);
        }
    }

    // Positive: match(true) fallthrough on a static property.
    public static function testMatchTrue(): string {
        return match (true) {
            self::$prop instanceof A, self::$prop instanceof B => get_class(self::$prop),
            default => 'none',
        };
    }
}

class Bar {
    private static A|int $a = 0;
    private static B|int $b = 0;

    // Negative: two different static-property receivers must not merge.
    public static function test(): void {
        if (self::$a instanceof A || self::$b instanceof B) {
            /** @mir-check self::$a is A|int */
            $x = self::$a;
            $_ = $x;
        }
    }
}
===expect===
