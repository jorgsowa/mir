===description===
wrong argument type via bare FQN instance method call is still caught
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
}
===expect===
App.php: InvalidArgument@4:17: Argument $n of process() expects 'int', got '"not-an-int"'
