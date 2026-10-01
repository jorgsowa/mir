===description===
A get-only hook parent property exposes no write contract, so a child may narrow its type
(plain, readonly or hooked), including via a bound template parent.
===config===
php_version=8.4
===file===
<?php
abstract class Id {
    abstract public int|string $value { get; }
}

final class PlainId extends Id {
    public string $value = 'a';
}

final class HookedId extends Id {
    public string $value { get => 'a'; }
}

final class PromotedId extends Id {
    public function __construct(public readonly string $value) {}
}

function read(PromotedId $i): string {
    $v = $i->value;
    /** @mir-check $v is string */
    return $v;
}
===expect===
