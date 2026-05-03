===description===
bracketed namespace use import not reported
===file===
<?php
namespace Outer {
    use Inner\Foo;
    function test(): void {
        new Foo();
    }
}
namespace Inner {
    class Foo {}
}
===expect===
===ignore===
TODO
