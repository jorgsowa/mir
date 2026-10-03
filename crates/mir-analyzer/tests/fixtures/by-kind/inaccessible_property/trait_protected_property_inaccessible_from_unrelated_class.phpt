===description===
Negative control for the K1 trait-composition fix: a protected trait
property must still be denied to a wholly unrelated class that neither
uses the trait nor extends a class that does.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                 ^^^^^^ InaccessibleProperty: Cannot access property Paging::$offset
    }
}
===expect===
