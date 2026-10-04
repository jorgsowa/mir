===description===
A class constant initialised by a concatenation is non-empty when either operand is a non-empty literal, even if the other operand cannot be resolved at collection time.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface HasSeparator {
    const SEPARATOR = '-';
}

class Keys {
    const PREFIX = 'k';
    const DOTTED = self::PREFIX . '.';
    const string TYPED = self::PREFIX . ':';
    const FROM_FUNCTION_LEFT = PHP_EOL . 'x';
    const FROM_FUNCTION_RIGHT = 'x' . PHP_EOL;
    const FROM_INTERFACE = HasSeparator::SEPARATOR . 'z';
    const LITERALS = 'a' . 'b';
    const EMPTY_LITERALS = '' . '';
    const BOTH_UNKNOWN = PHP_EOL . PHP_EOL;
    const NOT_CONCAT = PHP_EOL;

    public static function check(): void {
        $a = self::DOTTED;
        /** @mir-check $a is non-empty-string */
        $_a = $a;
        $b = self::TYPED;
        /** @mir-check $b is non-empty-string */
        $_b = $b;
        $c = self::FROM_FUNCTION_LEFT;
        /** @mir-check $c is non-empty-string */
        $_c = $c;
        $d = self::FROM_FUNCTION_RIGHT;
        /** @mir-check $d is non-empty-string */
        $_d = $d;
        $e = self::FROM_INTERFACE;
        /** @mir-check $e is non-empty-string */
        $_e = $e;
        $f = self::LITERALS;
        /** @mir-check $f is "ab" */
        $_f = $f;
        $g = self::EMPTY_LITERALS;
        /** @mir-check $g is "" */
        $_g = $g;
        $h = self::BOTH_UNKNOWN;
        /** @mir-check $h is mixed */
        $_h = $h;
        $i = self::NOT_CONCAT;
        /** @mir-check $i is mixed */
        $_i = $i;
    }

    /** @param non-empty-string $s */
    public static function need(string $s): void {}

    public static function use(): void {
        self::need(self::DOTTED);
        self::need(self::TYPED);
        self::need(self::FROM_INTERFACE);
    }

    public static function fillKeys(): void {
        $filled = array_fill_keys([self::DOTTED], 1);
        /** @mir-check $filled is non-empty-array<non-empty-string, 1> */
        $_filled = $filled;
        $_ = $filled;
    }
}
===expect===
