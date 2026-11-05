===description===
`unserialize` limited to no classes or a literal list can't reach a gadget, including options in a local. A variable class list proves nothing.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function none(): void {
    unserialize($_GET['x'], ['allowed_classes' => []]);
}

function listed(): void {
    unserialize($_GET['x'], ['allowed_classes' => ['A', B::class]]);
}

function local(): void {
    $options = ['allowed_classes' => false];
    unserialize($_GET['x'], $options);
}

function unknown(array $classes): void {
    unserialize($_GET['x'], ['allowed_classes' => $classes]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
}

function allowedAny(): void {
    unserialize($_GET['x'], ['allowed_classes' => true]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedUnserialize: Tainted unserialize() payload — possible object injection
}

class B {}
