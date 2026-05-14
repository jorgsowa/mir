===description===
new namespaced bare FQN without use statement produces no error
===file:Service.php===
<?php
namespace App;
class Service {
    public function run(): void {}
}
===file:Consumer.php===
<?php
function consume(): void {
    $s = new \App\Service();
    $s->run();
}
===expect===
