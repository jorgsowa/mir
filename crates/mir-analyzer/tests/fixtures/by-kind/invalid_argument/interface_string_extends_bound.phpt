===description===
interface-string<Child> satisfies a parameter typed interface-string<Parent>
when Child extends Parent.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Shape {}
interface Polygon extends Shape {}

/** @param interface-string<Shape> $className */
function needsShape(string $className) {
    return $className;
}

function forward(string $poly): void {
    /** @var interface-string<Polygon> $poly */
    needsShape($poly);
}
===expect===
