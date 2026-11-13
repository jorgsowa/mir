===description===
A bound violation names the parameter the template is inferred from, also for a named argument.
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

/**
 * @template T of Element
 * @param T $node
 */
function attach(string $label, Node $node): void {}

function run(Node $node): void {
    attach(node: $node, label: 'x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $node of attach() expects 'Element', got 'Node' — coercion may fail at runtime
}
