===description===
Static call on string
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public static function bar(): int {
        return 5;
    }
}
$foo = "A";
/** @suppress InvalidStringClass */
$b = $foo::bar();
//<^^^^^^^^^^^^^^^^ MixedAssignment: Variable $b is assigned a mixed type
===expect===
UnusedSuppress@9:0-9:0: Suppress annotation for 'InvalidStringClass' is never used
