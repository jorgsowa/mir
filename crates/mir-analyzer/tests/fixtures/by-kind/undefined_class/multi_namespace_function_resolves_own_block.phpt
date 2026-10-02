===description===
A function in a later namespace block resolves names against its own block: its own classes exist, another block's classes do not.
===file===
<?php
namespace A {
    class Real {}
}
namespace B {
    class Own {}
    function g(): void {
        new Own();
        new Missing();
//          ^^^^^^^ UndefinedClass: Class B\Missing does not exist
        new Real();
//          ^^^^ UndefinedClass: Class B\Real does not exist
    }
}
===expect===
