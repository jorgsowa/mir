===description===
Without `strict_types`, a `Stringable` object fits a `string` property like a `string` parameter.
===file===
<?php
final class M { public function __toString(): string { return 'x'; } }
class C {
    public string $a = '';
    /** @var string */
    protected $b = '';
    public function set(): void { $this->a = new M(); $this->b = new M(); }
}
