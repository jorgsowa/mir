===description===
int-mask-of<Flags::*> resolves a bare (unqualified) reference to the
declaring class's own short name, even when that class lives in a
namespace — not just `self`/`static`.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /**
     * @param int-mask-of<Flags::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

$f = new Flags();
$f->set(8);
===expect===
InvalidArgument@15:8-15:9: Argument $flags of set() expects '0|1|2|3', got '8'
