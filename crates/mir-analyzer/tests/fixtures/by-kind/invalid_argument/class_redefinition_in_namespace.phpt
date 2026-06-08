===description===
Class redefinition in namespace
===file===
<?php
namespace Aye {
    class Foo {}
    class Foo {}
}
===expect===
DuplicateClass@4:5-4:17: Class Aye\Foo has already been defined
