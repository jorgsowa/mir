===description===
Possibly unused param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @return void */
    public function foo(int $i) {}
}

(new A)->foo(4);
===expect===
