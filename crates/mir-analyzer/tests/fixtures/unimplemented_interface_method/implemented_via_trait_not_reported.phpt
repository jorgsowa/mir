===source===
<?php
interface Runnable {
    public function run(): void;
}
trait RunsTrait {
    public function run(): void {}
}
class Task implements Runnable {
    use RunsTrait;
}
===expect===
