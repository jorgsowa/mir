===description===
wrong argument type via bare FQN instance method call is still caught
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Processor.php===
<?php
class Processor {
    public function process(int $n): void { var_dump($n); }
}
===file:App.php===
<?php
function run(): void {
    $p = new \Processor();
    $p->process('not-an-int');
//              ^^^^^^^^^^^^ InvalidArgument: Argument $n of process() expects 'int', got '"not-an-int"'
}
===expect===
