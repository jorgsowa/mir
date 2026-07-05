===description===
int-mask-of<self::*> resolves against an enum's own literal-int `const`
declarations, the same way it does for classes and traits.
===config===
suppress=UnusedParam
===file===
<?php
enum Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /**
     * @param int-mask-of<self::*> $flags
     */
    public static function set(int $flags): void {}
}

Flags::set(8);
===expect===
InvalidArgument@12:11-12:12: Argument $flags of set() expects '0|1|2|3', got '8'
