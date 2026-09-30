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
//    ^^ ParseError: Parse error: expected ';' after expression
}
===expect===
