===description===
A readonly class implementing an interface get-only hook property is valid PHP 8.4
===file===
<?php
interface HasId {
    public int $id { get; }
}

final readonly class Ident implements HasId {
    public function __construct(public int $id) {}
}

final class Plain implements HasId {
    public function __construct(public readonly int $id) {}
}

function use_id(HasId $h): int {
    $viaInterface = $h->id;
    /** @mir-check $viaInterface is int */
    return $viaInterface;
}

function use_impl(Ident $i, Plain $p): int {
    $a = $i->id;
    $b = $p->id;
    /** @mir-check $a is int */
    /** @mir-check $b is int */
    return $a + $b;
}
