===file:Runnable.php===
<?php
interface Runnable {
    public function run(): void;
}
===file:RunsTrait.php===
<?php
trait RunsTrait {
    public function run(): void {}
}
===file:Worker.php===
<?php
class Worker implements Runnable {
    use RunsTrait;
}
===expect===
