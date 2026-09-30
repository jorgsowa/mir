===description===
reports this undefined method
===file===
<?php
class Svc {
    public function run(): void {
        $this->nonExistent();
//      ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Svc::nonExistent() does not exist
    }
}
===expect===
