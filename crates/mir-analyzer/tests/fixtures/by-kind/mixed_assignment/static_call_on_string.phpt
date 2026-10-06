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
UnusedSuppress@8:14-8:32: Suppress annotation for 'InvalidStringClass' is never used
