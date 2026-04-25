===file===
<?php
interface Runnable {
    public function run(): void;
}
class Task implements Runnable {
    public function run(): void {}
}
===expect===
