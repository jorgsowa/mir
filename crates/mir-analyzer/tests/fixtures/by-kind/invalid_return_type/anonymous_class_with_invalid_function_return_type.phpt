===description===
Anonymous class with invalid function return type
===file===
<?php
$foo = new class {
    public function a(): string {
        return 5;
    }
};
===expect===
InvalidReturnType@4:9-4:18: Return type '5' is not compatible with declared 'string'
