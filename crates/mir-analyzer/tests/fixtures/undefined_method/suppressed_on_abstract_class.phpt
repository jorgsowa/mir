===source===
<?php
abstract class Base {
    abstract public function run(): void;
}
function call_run(Base $b): void {
    $b->nonExistentMethod();
}
===expect===
