===description===
Incorrect callable param default
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function foo(callable $_a = "strlen"): void {}
}
