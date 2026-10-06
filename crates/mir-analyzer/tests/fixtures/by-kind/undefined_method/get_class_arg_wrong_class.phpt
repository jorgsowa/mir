===description===
Get class arg wrong class
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}

class B {}

$a = rand(0, 10) ? new A() : new B();

$a = match (get_class($a)) {
//   ^ +2:1 UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'string'
    A::class => $a->barBar(),
//              ^^^^^^^^^^^^ UndefinedMethod: Method A::barBar() does not exist
};
