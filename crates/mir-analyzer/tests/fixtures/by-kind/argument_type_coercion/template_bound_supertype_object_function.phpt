===description===
An object of the bound's parent class passed to a function template is a coercion; the bound class and its subclasses are accepted.
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
class Anchor extends Element {}

/**
 * @template T of Element
 * @param T $node
 * @return T
 */
function import(Node $node): Node { return $node; }

function run(Node $node, Element $element, Anchor $anchor): void {
    import($element);
    import($anchor);
    import($node);
//  ^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $node of import() expects 'Element', got 'Node' — coercion may fail at runtime
}
