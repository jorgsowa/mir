===description===
Negative control for the K1 trait-composition fix: a protected trait
property must still be denied to a wholly unrelated class that neither
uses the trait nor extends a class that does.
===config===
suppress=MissingConstructor
===file===
<?php

trait Paging {
    protected int $offset = 0;
}

class Listing {
    use Paging;
}

final class Unrelated {
    public function peek(Listing $l): int {
        return $l->offset;
    }
}
===expect===
InaccessibleProperty@13:19-13:25: Cannot access property Paging::$offset
