===source===
<?php
interface Runnable {
    public function run(): void;
}
trait RunsTrait {
    public function run(): void {}
}
abstract class Base implements Runnable {
    use RunsTrait;
}
class Task extends Base {}
===expect===

