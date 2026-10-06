===description===
Class redefinition in namespace
===file===
<?php
namespace Aye {
    class Foo {}
    class Foo {}
//  ^^^^^^^^^^^^ DuplicateClass: Class Aye\Foo has already been defined
}
