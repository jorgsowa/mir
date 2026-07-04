===description===
int-mask-of<static::FLAG_*> resolves the same way as `self::` — both refer
to the declaring class's own constants for this purpose.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /**
     * @param int-mask-of<static::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(3);   // A|B — valid
$f->set(4);   // out of range for {1, 2}
===expect===
InvalidArgument@14:8-14:9: Argument $flags of set() expects '0|1|2|3', got '4'
