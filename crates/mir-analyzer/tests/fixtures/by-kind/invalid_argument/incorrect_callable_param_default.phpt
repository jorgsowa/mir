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
//                                     ^^^^^^^^ CompileError: Cannot use string as default value for parameter $_a of type callable [default-value-type]
}
