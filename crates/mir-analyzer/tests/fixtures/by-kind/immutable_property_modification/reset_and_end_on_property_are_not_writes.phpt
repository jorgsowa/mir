===description===
`reset()` and `end()` only move an array's internal pointer, so passing a property isn't a write.
===file===
<?php
/** @psalm-immutable */
final class U {
    /** @param non-empty-array<string, int> $types */
    public function __construct(public array $types) {}
    public function first(): int { return reset($this->types); }
    public function last(): int { return end($this->types); }
    public function sorted(): void { sort($this->types); }
//                                        ^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property types of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
}

/** @psalm-pure */
function head(U $u): int { return reset($u->types); }
