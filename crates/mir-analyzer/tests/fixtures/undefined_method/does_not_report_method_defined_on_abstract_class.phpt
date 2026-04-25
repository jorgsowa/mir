===file===
<?php
abstract class Base {
    abstract public function run(): void;
}
function f(Base $b): void {
    $b->run();
}
===expect===
