===description===
int-mask-of<self::FLAG_*> rejects a literal integer that cannot be formed by
OR-ing any subset of the matched constants {1, 2, 4}: 8 is out of range.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;
    const FLAG_C = 4;

    /**
     * @param int-mask-of<self::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(8);
===expect===
InvalidArgument@14:8-14:9: Argument $flags of set() expects '0|1|2|3|4|5|6|7', got '8'
