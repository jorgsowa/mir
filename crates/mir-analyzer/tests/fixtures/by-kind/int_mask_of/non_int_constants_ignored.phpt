===description===
int-mask-of<self::FLAG_*> only pulls in constants that are literal ints; a
same-prefixed string constant is silently excluded from the mask rather than
breaking resolution.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;
    const FLAG_LABEL = "flags";

    /**
     * @param int-mask-of<self::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(3);
$f->set(4);
===expect===
InvalidArgument@15:8-15:9: Argument $flags of set() expects '0|1|2|3', got '4'
