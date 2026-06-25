===description===
self::INT_CONST used as array key does not emit InvalidArrayOffset or MixedArrayOffset.
After the fix, self::CONST returns the literal int type instead of mixed.
===config===
suppress=UnusedVariable
===file===
<?php
class Flags {
    const FLAG_A = 1;
    const FLAG_B = 2;

    /** @var array<int, string> */
    private array $map = [];

    public function store(string $v): void {
        $this->map[self::FLAG_A] = $v;
        $this->map[self::FLAG_B] = $v;
    }
}
===expect===
