===description===
`instanceof` narrowing on a `TIntersection` atom (e.g. `Box<int>&Quacks`,
formed by an earlier unrelated instanceof check) projects a generic
part's own type params onto the new subclass, mirroring the
non-intersection case — previously it always appended a raw,
empty-type-params atom to the intersection.
===config===
suppress=UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    private $value;

    /** @param T $value */
    public function __construct($value) {
        $this->value = $value;
    }
}

class SubBox extends Box {}
interface Quacks {}

/** @param Box<int> $x */
function narrowsIntersection($x): void {
    if ($x instanceof Quacks) {
        if ($x instanceof SubBox) {
            /** @mir-check $x is Box<int>&Quacks&SubBox<int> */
            $_ = $x;
        }
    }
}
===expect===
