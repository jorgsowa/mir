===file:Runnable.php===
<?php
interface Runnable {
    public function run(): void;
    public function stop(): void;
}
===file:Task.php===
<?php
class Task implements Runnable {
    public function run(): void {}
}
===expect===
Task.php: UnimplementedInterfaceMethod: Class Task must implement Runnable::stop() from interface
