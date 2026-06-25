===description===
Two classes with the same short name in different namespaces are distinct — no DuplicateClass.
===file===
<?php
namespace App;
class User {}

namespace Other;
class User {}
===expect===
