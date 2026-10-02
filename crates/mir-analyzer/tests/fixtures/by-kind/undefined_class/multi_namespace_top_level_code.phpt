===description===
Top-level code inside each namespace block is analyzed against that block's namespace and imports.
===file===
<?php
namespace A {
    class Real {}
    new Real();
}
namespace B {
    use A\Real as R;
    class Own {}
    new Own();
    new R();
    new Missing();
//      ^^^^^^^ UndefinedClass: Class B\Missing does not exist
    new \B\Nope();
//      ^^^^^^^ UndefinedClass: Class B\Nope does not exist
    undefined_fn();
//  ^^^^^^^^^^^^^^ UndefinedFunction: Function undefined_fn() is not defined
}
===expect===
