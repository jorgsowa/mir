===description===
class_exists with string literal arg suppresses UndefinedClass in true branch
===file===
<?php
function test(): void {
    if (class_exists('Optional\Pkg')) {
        new \Optional\Pkg();
    }
}
===expect===
