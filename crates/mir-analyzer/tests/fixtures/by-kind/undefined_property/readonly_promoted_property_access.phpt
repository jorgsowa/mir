===description===
Readonly promoted property access
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function __construct(private readonly string $bar) {
    }
}

$a = new A("hello");
$b = $a->bar;
//       ^^^ InaccessibleProperty: Cannot access property A::$bar
===expect===
