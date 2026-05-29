===description===
cross file missing method
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
Task.php: UnimplementedInterfaceMethod@2:0-2:32: Class Task must implement Runnable::stop() from interface
