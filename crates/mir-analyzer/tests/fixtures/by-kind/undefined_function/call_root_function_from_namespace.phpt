===description===
Call root function from namespace
===ignore===
TODO
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
ParseError@9:7-9:9: Parse error: expected ';' after expression
