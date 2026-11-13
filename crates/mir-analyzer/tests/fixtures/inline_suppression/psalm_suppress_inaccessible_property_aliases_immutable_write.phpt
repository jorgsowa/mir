===description===
Psalm's `InaccessibleProperty` and `ImpurePropertyAssignment` name a write to an
immutable object's property, which mir reports as `ImmutablePropertyModification`.
Either name suppresses it, and a trailing reason is not read as a kind.
===file===
<?php
/** @psalm-immutable */
final class Loc {
    public bool $done = false;
    public int $start = 0;

    /**
     * @psalm-external-mutation-free
     * @psalm-suppress InaccessibleProperty caching
     */
    private function calc(): void { $this->done = true; }

    /** @psalm-suppress ImpurePropertyAssignment */
    private function reset(): void { $this->start = 0; }
}

/** @psalm-suppress InaccessibleProperty */
function move(Loc $l): void { $l->start = 1; }

function unsuppressed(Loc $l): void { $l->start = 2; }
//                                    ^^^^^^^^^^^^^ ImmutablePropertyModification: Assigning to property start of $l in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
