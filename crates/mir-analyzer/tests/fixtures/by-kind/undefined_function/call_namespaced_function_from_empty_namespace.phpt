===description===
Call namespaced function from empty namespace
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
