===description===
Negative control for the K1 trait-composition fix: a private trait
property becomes private TO THE CONSUMING CLASS specifically — same as if
that class had declared it itself — so a further subclass (which doesn't
itself `use` the trait) must still be denied access, exactly like a
private property declared directly on the parent.
===config===
suppress=MissingConstructor
===file===
<?php

trait Paging {
    private int $limit = 10;
}

class Listing {
    use Paging;
}

final class SpecialListing extends Listing {
    public function limit(): int {
        return $this->limit;
    }
}
===expect===
InaccessibleProperty@13:22-13:27: Cannot access property Paging::$limit
