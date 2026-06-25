===description===
Two traits with the same short name in different namespaces are distinct — no DuplicateTrait.
===file===
<?php
namespace App;
trait Timestampable {}

namespace Other;
trait Timestampable {}
===expect===
