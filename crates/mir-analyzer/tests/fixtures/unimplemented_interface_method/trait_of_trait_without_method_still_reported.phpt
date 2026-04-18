===source===
<?php
interface Runnable {
    public function run(): void;
}
trait HelperTrait {
    public function helper(): void {}
}
trait RunsTrait {
    use HelperTrait;
    // run() is not provided anywhere in the chain
}
class Task implements Runnable {
    use RunsTrait;
}
===expect===
UnimplementedInterfaceMethod: class Task implements Runnable {
