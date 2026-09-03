===description===
`collector/interface.rs` and `collector/trait.rs` built their
`@psalm-type` alias map AFTER the member loop and passed `None` into
`build_method_storage` during that loop, unlike `class.rs` which hoists
the alias map first -- a class-level type alias referenced from a
method on the same interface/trait failed to expand and resolved as a
bogus undefined-class reference instead.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type UserId = int
 */
interface HasId {
    /** @return UserId */
    public function getId();
}

function useId(HasId $h): int {
    $id = $h->getId();
    /** @mir-check $id is int */
    $_ = 1;
    return $id;
}

/**
 * @psalm-type Meters = float
 */
trait HasLength {
    /** @return Meters */
    public function getLength() {
        return 1.0;
    }
}

class Track {
    use HasLength;
}

function useLength(Track $t): float {
    return $t->getLength();
}
===expect===
