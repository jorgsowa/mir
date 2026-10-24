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
    \foo();
}
