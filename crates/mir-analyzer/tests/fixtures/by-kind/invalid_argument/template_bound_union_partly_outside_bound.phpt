===description===
A union with a member that can never satisfy the bound stays an invalid argument, even when another member is the bound's supertype.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Node {}
class Element extends Node {}
class Comment {}

/**
 * @template T of Element
 * @param T $node
 */
function import(object $node): void {}

function run(Node|Comment $value): void {
    import($value);
//  ^^^^^^^^^^^^^^ InvalidArgument: Argument $node of import() expects 'Element', got 'Node|Comment'
}
