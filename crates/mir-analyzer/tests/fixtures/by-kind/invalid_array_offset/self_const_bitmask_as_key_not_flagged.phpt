===description===
Bitwise OR of self::INT_CONST values used as an array key does not flag.
The OR of two literal ints produces an int, which is a valid array-key.
===config===
suppress=UnusedVariable
===file===
<?php
class Permission {
    const READ    = 1;
    const WRITE   = 2;
    const EXECUTE = 4;

    /** @var array<int, string> */
    private array $labels = [];

    public function label(): void {
        $rw  = self::READ | self::WRITE;
        $rwx = self::READ | self::WRITE | self::EXECUTE;
        $this->labels[$rw]  = 'read-write';
        $this->labels[$rwx] = 'read-write-execute';
    }
}
===expect===
