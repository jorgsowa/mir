===description===
A class unrelated to a function template's bound is an invalid argument.
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
class Comment {}

/**
 * @template T of Element
 * @param T $node
 */
function import(object $node): void {}

function run(Comment $comment): void {
    import($comment);
//  ^^^^^^^^^^^^^^^^ InvalidArgument: Argument $node of import() expects 'Element', got 'Comment'
}
