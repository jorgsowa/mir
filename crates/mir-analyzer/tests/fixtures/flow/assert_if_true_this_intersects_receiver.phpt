===description===
`@phpstan-assert-if-true X $this` narrows the receiver like `instanceof X`: an
interface is intersected with the receiver's class, and a subclass replaces it.
===file===
<?php
interface ManyToMany {}

abstract class Mapping {
    /** @phpstan-assert-if-true ManyToMany $this */
    public function isManyToMany(): bool { return $this instanceof ManyToMany; }

    /** @phpstan-assert-if-true Owning $this */
    public function isOwning(): bool { return $this instanceof Owning; }

    public function name(): string { return ''; }
}

abstract class Owning extends Mapping {}

/** @param Mapping&ManyToMany $m */
function take(Mapping $m): void { echo $m->name(); }

function viaAssert(Mapping $m): void {
    assert($m->isManyToMany());
    take($m);
    echo $m->name();
}

function viaIf(Mapping $m): void {
    if ($m->isManyToMany()) {
        /** @mir-check $m is Mapping&ManyToMany */
        take($m);
    }
}

function subclass(Mapping $m): void {
    if ($m->isOwning()) {
        /** @mir-check $m is Owning */
        echo $m->name();
    }
}
