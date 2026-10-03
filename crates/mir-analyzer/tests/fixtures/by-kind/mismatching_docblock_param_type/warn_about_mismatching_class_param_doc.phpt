===description===
Warn about mismatching class param doc
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}

class X {
    /**
     * @param B $class
     */
    public function boo(A $class): void {}
}
===expect===
