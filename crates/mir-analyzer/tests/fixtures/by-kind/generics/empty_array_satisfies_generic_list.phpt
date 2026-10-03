===description===
An empty array literal satisfies a generic list/array type-argument, but a wrong
non-empty type-argument is still reported (generics stay invariant otherwise).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
final class Box {
    /** @param T $v */
    public function __construct(public mixed $v) {}
}

/** @return Box<list<int>> */
function ok(): Box {
    return new Box([]);
}

/** @return Box<array<string, int>> */
function ok_assoc(): Box {
    return new Box([]);
}

/** @return Box<string> */
function wrong(): Box {
    return new Box(5);
//  ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Box<int>' is not compatible with declared 'Box<string>'
}

===expect===
