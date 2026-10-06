===description===
MagicMethodParamTypesCheckedForClasses
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A
{
    public function a(int $className): int { return 0; }
}

/**
 * @method int a(string $a)
// ^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::a() signature mismatch: parameter $a type 'string' is incompatible with parent type 'int'
 */
class B extends A {}
