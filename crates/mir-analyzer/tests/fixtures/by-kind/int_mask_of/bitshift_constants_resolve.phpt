===description===
int-mask-of<self::FLAG_*> resolves constants declared with a bit-shift
expression (`1 << 0`), the idiomatic way to declare bitflags, not just bare
integer literals.
===config===
suppress=UnusedParam
===file===
<?php
class Flags {
    const FLAG_A = 1 << 0;
    const FLAG_B = 1 << 1;

    /**
     * @param int-mask-of<self::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(3);
$f->set(4);
===expect===
InvalidArgument@14:8-14:9: Argument $flags of set() expects '0|1|2|3', got '4'
