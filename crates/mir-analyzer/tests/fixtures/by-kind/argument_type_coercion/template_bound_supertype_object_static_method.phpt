===description===
An object of the bound's parent class passed to a static method template is a coercion.
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

final class Registry {
    /**
     * @template T of Element
     * @param T $node
     */
    public static function register(Node $node): void {}
}

function run(Node $node, Element $element): void {
    Registry::register($element);
    Registry::register($node);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $node of register() expects 'Element', got 'Node' — coercion may fail at runtime
}
