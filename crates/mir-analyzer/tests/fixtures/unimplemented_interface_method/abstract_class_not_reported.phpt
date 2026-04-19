===source===
<?php
interface Runnable {
    public function run(): void;
}
abstract class AbstractTask implements Runnable {}
===expect===
