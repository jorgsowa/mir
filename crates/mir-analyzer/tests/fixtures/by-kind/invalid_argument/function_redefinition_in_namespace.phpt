===description===
Function redefinition in same namespace
===file===
<?php
namespace Aye {
    function foo(): void {}
    function foo(): void {}
}
===expect===
DuplicateFunction@4:4-4:27: Function Aye\foo() has already been defined
