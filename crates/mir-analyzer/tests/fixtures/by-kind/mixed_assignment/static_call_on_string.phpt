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
//            ^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'InvalidStringClass' is never used
$b = $foo::bar();
//<^^^^^^^^^^^^^^^^ MixedAssignment: Variable $b is assigned a mixed type
