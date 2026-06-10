===description===
Interface redefinition in namespace
===file===
<?php
namespace Aye {
    interface Foo {}
    interface Foo {}
}
===expect===
DuplicateInterface@4:5-4:21: Interface Aye\Foo has already been defined
