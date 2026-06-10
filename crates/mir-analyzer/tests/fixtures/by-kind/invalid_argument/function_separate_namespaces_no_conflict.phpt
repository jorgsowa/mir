===description===
Functions with the same name in separate namespaces do not conflict
===file===
<?php
namespace Aye {
    function foo(): void {}
}
namespace Bee {
    function foo(): void {}
}
===expect===
