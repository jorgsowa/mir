===description===
Function redefinition in same namespace
===file===
<?php
namespace Aye {
    function foo(): void {}
    function foo(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^ DuplicateFunction: Function Aye\foo() has already been defined
}
===expect===
