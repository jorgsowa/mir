===description===
callRootFunctionFromNamespace
===file===
<?php
namespace {
    /** @return void */
    function foo() {

    }
}
namespace A {
    Aoo();
}
===expect===
UndefinedFunction
===ignore===
TODO
