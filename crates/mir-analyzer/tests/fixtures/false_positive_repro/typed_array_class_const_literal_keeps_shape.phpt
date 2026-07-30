===description===
A PHP 8.3 typed class constant with a bare `array` hint (`public const
array D = [...]`) must not lose its literal element count/types — the
native hint previously always won over literal inference (same underlying
priority bug as A3/B7, just for arrays instead of scalars), so
`array_key_first()` on a 4-element constant looked like it could return
null.
===file===
<?php
final class C {
    public const array D = [',', ';', "\t", '|'];
    public static function f(): string {
        return array_key_first(self::D);
    }
}
===expect===
