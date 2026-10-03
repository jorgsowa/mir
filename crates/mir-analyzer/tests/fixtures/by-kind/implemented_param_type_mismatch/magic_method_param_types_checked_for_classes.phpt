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
 */
class B extends A {}

===expect===
MethodSignatureMismatch@8:3-8:27: Method B::a() signature mismatch: parameter $a type 'string' is incompatible with parent type 'int'
