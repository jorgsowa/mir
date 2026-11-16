===description===
From PHP 8.6 a readonly property may declare a default value, also in a readonly class.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
class A {
    public readonly string $s = "a";
    public readonly ?int $n = null;
}

readonly class B {
    public int $count = 0;
}

function read(A $a, B $b): string {
    return $a->s . $b->count . ($a->n ?? 0);
}
