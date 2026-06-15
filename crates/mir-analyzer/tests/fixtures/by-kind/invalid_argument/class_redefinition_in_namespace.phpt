===description===
Class redefinition in namespace
===file===
<?php
namespace Aye {
    class Foo {}
    class Foo {}
}
===expect===
DuplicateClass@4:4-4:16: Class Aye\Foo has already been defined
