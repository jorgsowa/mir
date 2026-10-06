===description===
trait of trait without method still reported
===file===
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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Task must implement Runnable::run() from interface
    use RunsTrait;
}
