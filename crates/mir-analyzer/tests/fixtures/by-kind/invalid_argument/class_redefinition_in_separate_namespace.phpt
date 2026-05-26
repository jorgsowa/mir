===description===
Class redefinition in separate namespace
===file===
<?php
namespace Aye {
    class Foo {}
}
namespace Aye {
    class Foo {}
}
===expect===
DuplicateClass
===ignore===
TODO
