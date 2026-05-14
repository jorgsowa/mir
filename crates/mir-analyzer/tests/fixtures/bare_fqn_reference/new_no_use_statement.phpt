===description===
new bare FQN without use statement produces no error
===file:Service.php===
<?php
class Service {
    public function run(): void {}
}
===file:Consumer.php===
<?php
function consume(): void {
    $s = new \Service();
    $s->run();
}
===expect===
