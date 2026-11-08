===description===
The result of a method declared `function &m()` can be passed to a by-reference parameter.
===file===
<?php
class NA {
    public static function &get(array &$a): array { return $a; }
    public static function take(array &$a): int { return count($a); }
}
class FS {
    private array $v = [];
    public function &getValues(): array { return $this->v; }
    public function copy(): array { return $this->v; }
    public function m(): void { NA::take($this->getValues()); }
}
function f(FS $fs, array $x): void {
    NA::take($fs->getValues());
    NA::take(NA::get($x));
    NA::take($fs->copy());
//           ^^^^^^^^^^^ InvalidPassByReference: Argument $a of take() must be passed by reference
}
