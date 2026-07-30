===description===
A class constant's array literal (no native type hint at all) must get its
key/value shape inferred from the literal, not widened to a bare `array`
— accessing a known literal key must not be flagged, while an unknown key
still is (covered by the negative control).
===file===
<?php
final class C {
    public const MAP = ['a' => 1, 'b' => 2];
    public static function f(): int {
        return self::MAP['a'];
    }
}
===expect===
