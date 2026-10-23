===description===
passing an object of a class in an inheritance cycle from an immutable method terminates
===config===
<mir>
  <issueHandlers>
    <ImpureFunctionCall errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A extends B {}
class B extends A {}
//<^^^^^^^^^^^^^^^^^^^^ CircularInheritance: Class B has a circular inheritance chain
function sink(object $o): void {}
/** @psalm-immutable */
final class Holder {
    public function __construct(private A $a) {}
    public function go(): void { sink($this->a); }
}
