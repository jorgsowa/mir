===description===
callNamespacedFunctionFromEmptyNamespace
===file===
<?php
namespace A {
    /** @return void */
    function foo() {

    }
}
namespace {
    foo();
}
===expect===
UndefinedFunction
===ignore===
TODO
