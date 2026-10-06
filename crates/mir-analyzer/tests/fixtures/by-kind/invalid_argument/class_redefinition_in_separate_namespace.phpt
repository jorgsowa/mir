===description===
Class redefinition in separate namespace
===file===
<?php
namespace Aye {
    class Foo {}
}
namespace Aye {
    class Foo {}
//  ^^^^^^^^^^^^ DuplicateClass: Class Aye\Foo has already been defined
}
