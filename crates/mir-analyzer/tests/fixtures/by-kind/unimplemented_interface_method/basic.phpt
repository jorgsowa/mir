===description===
Basic
===file===
<?php
interface Runnable {
    public function run(): void;
}
class Task implements Runnable {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Task must implement Runnable::run() from interface
