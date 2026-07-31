===description===
FP-K1: a private/protected property declared on a TRAIT was flagged
inaccessible from the class that `use`s it — property_inaccessible
compared self_fqcn against the trait's own FQCN (the reported owner),
with no awareness that a trait's members are copy-pasted into every
consuming class, unlike normal inheritance.
===config===
suppress=MissingConstructor
===file===
<?php

trait Paging {
    private int $limit = 10;
    protected int $offset = 0;
}

final class Listing {
    use Paging;

    public function limit(): int {
        return $this->limit;
    }

    public function offset(): int {
        return $this->offset;
    }
}
===expect===
