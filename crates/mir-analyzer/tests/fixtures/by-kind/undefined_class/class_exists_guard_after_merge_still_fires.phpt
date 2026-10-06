===description===
UndefinedClass still fires after the class_exists if block ends (guard does not escape)
===file===
<?php
function test(): void {
    if (class_exists(\Optional\Pkg::class)) {
        // fine inside
    }
    new \Optional\Pkg();
//      ^^^^^^^^^^^^^ UndefinedClass: Class Optional\Pkg does not exist
}
