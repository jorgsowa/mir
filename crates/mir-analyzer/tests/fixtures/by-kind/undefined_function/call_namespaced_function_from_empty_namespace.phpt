===description===
Calling a namespaced function from the global namespace block without importing it is undefined
===file===
<?php
namespace A {
    /** @return void */
    function foo() {

    }
}
namespace {
    foo();
//  ^^^^^ UndefinedFunction: Function foo() is not defined
}
===expect===
