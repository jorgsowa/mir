===description===
int-mask-of<self::FLAG_*> resolves against a trait's own constants when the
method is declared directly on the trait.
===config===
suppress=UnusedParam
===file===
<?php
trait HasFlags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /**
     * @param int-mask-of<self::FLAG_*> $flags
     */
    public function set(int $flags): void {}
}

class Flags {
    use HasFlags;
}

$f = new Flags();
$f->set(3);
$f->set(4);
===expect===
InvalidArgument@18:8-18:9: Argument $flags of set() expects '0|1|2|3', got '4'
