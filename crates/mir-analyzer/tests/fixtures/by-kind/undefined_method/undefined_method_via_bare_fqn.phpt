===description===
calling an undefined method via bare FQN is still caught
===file:Service.php===
<?php
class Service {
    public function run(): void {}
}
===file:Consumer.php===
<?php
function consume(): void {
    $s = new \Service();
    $s->nonexistent();
}
===expect===
Consumer.php: UndefinedMethod@4:5-4:22: Method Service::nonexistent() does not exist
