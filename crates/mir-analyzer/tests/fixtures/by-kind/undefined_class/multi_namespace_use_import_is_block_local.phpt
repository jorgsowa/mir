===description===
A `use` import applies only to its own namespace block.
===file===
<?php
namespace Lib {
    class Tool {}
}
namespace One {
    use Lib\Tool;
    function f(): void { new Tool(); }
}
namespace Two {
    function g(): void {
        new Tool();
//          ^^^^ UndefinedClass: Class Two\Tool does not exist
    }
}
===expect===
