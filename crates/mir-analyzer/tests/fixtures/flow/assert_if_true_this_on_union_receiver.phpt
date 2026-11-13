===description===
An `@phpstan-assert-if-*` on `$this` still narrows a receiver that an earlier check
left as a union of related classes, since every member inherits the same method.
On a union of unrelated classes it doesn't apply.
===file===
<?php
abstract class Mapping {
    /**
     * @phpstan-assert-if-true Owning $this
     * @phpstan-assert-if-false Inverse $this
     */
    final public function isOwningSide(): bool { return $this instanceof Owning; }

    /** @phpstan-assert-if-true ManyToOne $this */
    final public function isManyToOne(): bool { return $this instanceof ManyToOne; }
}

abstract class Owning extends Mapping {}
final class ManyToOne extends Owning {}
final class Inverse extends Mapping { public string $mappedBy = ''; }

function afterEarlierCheck(Mapping $m): string {
    if ($m->isManyToOne()) {
        echo 'x';
    }
    if (!$m->isOwningSide()) {
        return $m->mappedBy;
    }
    return '';
}

final class Other {
    /** @phpstan-assert-if-true Inverse $this */
    public function isOwningSide(): bool { return false; }
}

function unrelated(ManyToOne|Other $m): void {
    if ($m->isOwningSide()) {
        /** @mir-check $m is ManyToOne|Other */
        echo 1;
    }
}
