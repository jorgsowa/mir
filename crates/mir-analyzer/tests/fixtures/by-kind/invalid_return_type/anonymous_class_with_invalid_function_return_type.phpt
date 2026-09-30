===description===
Anonymous class with invalid function return type
===config===
suppress=UnusedVariable
===file===
<?php
$foo = new class {
    public function a(): string {
        return 5;
//      ^^^^^^^^^ InvalidReturnType: Return type '5' is not compatible with declared 'string'
    }
};
===expect===
