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
DuplicateClass@6:5-6:17: Class Aye\Foo has already been defined
