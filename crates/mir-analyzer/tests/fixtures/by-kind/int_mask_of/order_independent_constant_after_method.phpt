===description===
int-mask-of<self::*> resolution does not depend on source order — a
constant declared *after* the method that references it via `self::` still
resolves, since PHP itself allows forward references to class constants.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    /**
     * @param int-mask-of<self::FLAG_*> $flags
     */
    public function set(int $flags): void {}

    const FLAG_A = 1;
    const FLAG_B = 2;
}

$f = new Flags();
$f->set(3);
$f->set(4);
===expect===
InvalidArgument@14:8-14:9: Argument $flags of set() expects '0|1|2|3', got '4'
