===description===
parent::INT_CONST used as array key does not emit InvalidArrayOffset.
parent:: resolves to the parent class and returns the literal int type.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {
    const SLOT = 7;
}
class Child extends Base {
    /** @var array<int, string> */
    private array $cache = [];

    public function save(string $v): void {
        $this->cache[parent::SLOT] = $v;
    }
}
===expect===
