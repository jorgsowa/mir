===description===
String concatenation of literals folds to a literal result: "foo" . "bar" = "foobar"
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $a = "Hello" . " " . "World";
    /** @mir-check $a is "Hello World" */
    $_ = $a;

    $b = "user_" . "name";
    /** @mir-check $b is "user_name" */
    $_ = $b;

    $c = "count: " . 42;
    /** @mir-check $c is "count: 42" */
    $_ = $c;

    $d = "" . "hello";
    /** @mir-check $d is "hello" */
    $_ = $d;
}
===expect===
