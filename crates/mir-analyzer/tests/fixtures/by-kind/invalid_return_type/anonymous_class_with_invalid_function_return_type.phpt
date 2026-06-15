===description===
Anonymous class with invalid function return type
===config===
suppress=UnusedVariable
===file===
<?php
$foo = new class {
    public function a(): string {
        return 5;
    }
};
===expect===
InvalidReturnType@4:8-4:17: Return type '5' is not compatible with declared 'string'
