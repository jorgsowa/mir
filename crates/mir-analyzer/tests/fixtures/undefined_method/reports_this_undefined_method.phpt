===source===
<?php
class Svc {
    public function run(): void {
        $this->nonExistent();
    }
}
===expect===
UndefinedMethod: Method Svc::nonExistent() does not exist
