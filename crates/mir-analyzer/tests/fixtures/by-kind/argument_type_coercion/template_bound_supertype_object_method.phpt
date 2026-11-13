===description===
An object of the bound's parent class passed to an instance method template is a coercion.
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

final class Document {
    /**
     * @template T of Element
     * @param T $node
     * @return T
     */
    public function adopt(Node $node): Node { return $node; }
}

function run(Document $doc, Node $node, Element $element): void {
    $doc->adopt($element);
    $doc->adopt($node);
//  ^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $node of adopt() expects 'Element', got 'Node' — coercion may fail at runtime
}
