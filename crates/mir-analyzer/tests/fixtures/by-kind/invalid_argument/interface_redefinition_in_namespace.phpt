===description===
Interface redefinition in namespace
===file===
<?php
namespace Aye {
    interface Foo {}
    interface Foo {}
//  ^^^^^^^^^^^^^^^^ DuplicateInterface: Interface Aye\Foo has already been defined
}
===expect===
