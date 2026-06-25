===description===
Two interfaces with the same short name in different namespaces are distinct — no DuplicateInterface.
===file===
<?php
namespace App;
interface Repository {}

namespace Other;
interface Repository {}
===expect===
