===description===
`new` keeps a literal template argument when the template's bound only admits literals
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T of 'a'|'b' */
final class Tree {
    /** @param T $type */
    public function __construct(string $type) {}
}

/** @template T */
final class Box {
    /** @param T $value */
    public function __construct(mixed $value) {}
}

/** @return Tree<'a'> */
function tree(): Tree {
    $box = new Box(5);
    /** @mir-check $box is Box<int> */
    return new Tree('a');
}
