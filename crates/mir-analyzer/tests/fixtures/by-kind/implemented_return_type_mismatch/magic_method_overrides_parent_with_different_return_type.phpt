===description===
Magic method overrides parent with different return type
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class C {}
class D {}

class A {
    public function foo(string $s) : C {
        return new C;
    }
}

/** @method D foo(string $s) */
//  ^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foo() signature mismatch: return type 'D' is not a subtype of parent 'C'
class B extends A {}
===expect===
