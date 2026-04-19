===source===
<?php
interface Runnable {
    public function run(): void;
}
trait ActualRunner {
    public function run(): void {}
}
trait RunsTrait {
    use ActualRunner;
}
abstract class Base implements Runnable {
    use RunsTrait;
}
class Task extends Base {}
===expect===
