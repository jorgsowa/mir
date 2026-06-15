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
DuplicateClass@6:4-6:16: Class Aye\Foo has already been defined
