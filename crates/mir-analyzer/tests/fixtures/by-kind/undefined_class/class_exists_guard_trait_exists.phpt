===description===
trait_exists guard suppresses UndefinedClass in true branch
===file===
<?php
function test(): void {
    if (trait_exists(\Optional\MyTrait::class)) {
        new class {
            use \Optional\MyTrait;
        };
    }
}
===expect===
