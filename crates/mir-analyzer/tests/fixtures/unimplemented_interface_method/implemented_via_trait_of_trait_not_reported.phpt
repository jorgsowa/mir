===description===
implemented via trait of trait not reported
===file===
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
class Task implements Runnable {
    use RunsTrait;
}
===expect===
===ignore===
TODO
