===description===
Different argument name
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function fooFoo(int $a): void {

    }
}

class B extends A {
    public function fooFoo(int $b): void {

    }
}
===expect===
