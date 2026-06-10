===description===
Call namespaced function from empty namespace
===ignore===
TODO
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
