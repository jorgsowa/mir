===description===
int-mask-of<self::FLAG_*> resolves against the class's own literal-int
constants matching the `FLAG_` prefix and expands to all 8 OR-combinations
of {1, 2, 4}, same as writing int-mask<1, 2, 4> by hand.
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
$f->set(0);  // no flags
$f->set(1);  // A
$f->set(2);  // B
$f->set(3);  // A|B
$f->set(4);  // C
$f->set(5);  // A|C
$f->set(6);  // B|C
$f->set(7);  // A|B|C
===expect===
