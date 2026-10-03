===description===
Magic method overrides parent with different param type
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class C {}
class D extends C {}

class A {
    public function foo(string $s) : C {
        return new C;
    }
}

/** @method D foo(int $s) */
class B extends A {}
===expect===
MethodSignatureMismatch@11:4-11:28: Method B::foo() signature mismatch: parameter $s type 'int' is incompatible with parent type 'string'
