===description===
Anonymous class with bad statement
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$foo = new class {
    public function a() {
        new B();
//          ^ UndefinedClass: Class B does not exist
    }
};
