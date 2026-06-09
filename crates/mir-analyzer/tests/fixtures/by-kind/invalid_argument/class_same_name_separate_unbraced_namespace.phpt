===description===
Same class name in separate unbraced namespaces is not a redefinition
===file===
<?php
namespace A;
class Foo {}

namespace B;
class Foo {}
===expect===
