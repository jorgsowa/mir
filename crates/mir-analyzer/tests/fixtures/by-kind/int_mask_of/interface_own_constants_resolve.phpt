===description===
int-mask-of<self::*> resolves against an interface's own literal-int
constants when referenced from a method declared on that same interface.
===config===
suppress=UnusedParam
===file===
<?php
interface HasFlags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /**
     * @param int-mask-of<self::*> $flags
     */
    public function set(int $flags): void;
}

class Flags implements HasFlags {
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(8);
===expect===
