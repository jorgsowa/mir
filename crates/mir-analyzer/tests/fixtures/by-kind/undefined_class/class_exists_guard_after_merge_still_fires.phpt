===description===
UndefinedClass still fires after the class_exists if block ends (guard does not escape)
===file===
<?php
function test(): void {
    if (class_exists(\Optional\Pkg::class)) {
        // fine inside
    }
    new \Optional\Pkg();
}
===expect===
UndefinedClass@6:9-6:22: Class Optional\Pkg does not exist
