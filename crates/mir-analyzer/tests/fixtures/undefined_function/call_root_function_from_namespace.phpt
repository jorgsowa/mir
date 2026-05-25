===description===
Call root function from namespace
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
