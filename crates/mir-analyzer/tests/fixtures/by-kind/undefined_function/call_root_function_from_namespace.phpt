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
ParseError@9:6-9:8: Parse error: expected ';' after expression
