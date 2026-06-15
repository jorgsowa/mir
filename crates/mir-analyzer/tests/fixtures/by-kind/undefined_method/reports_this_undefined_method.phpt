===description===
reports this undefined method
===file===
<?php
class Svc {
    public function run(): void {
        $this->nonExistent();
    }
}
===expect===
UndefinedMethod@4:8-4:28: Method Svc::nonExistent() does not exist
