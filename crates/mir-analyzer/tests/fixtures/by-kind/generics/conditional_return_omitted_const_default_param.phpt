===description===
An omitted argument defaulting to `self::CONST` resolves a `$param` conditional `@return` on a static method
===file===
<?php
final class Box {
    public const STRICT = 1;

    /** @return ($mode is 1 ? int : null) */
    public static function make(int $mode = self::STRICT): ?int { return $mode === 1 ? 1 : null; }
}

$a = Box::make();
/** @mir-check $a is int */
echo $a;
