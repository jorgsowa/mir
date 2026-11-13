===description===
An object of an unrelated class is an invalid shape value; a subclass is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Element {}
class Anchor extends Element {}
class Comment {}

/** @return array{node: Element} */
function bad(Comment $comment): array {
    return ['node' => $comment];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'node': Comment}' is not compatible with declared 'array{'node': Element}'
}

/** @return array{node: Element} */
function good(Anchor $anchor): array {
    return ['node' => $anchor];
}
