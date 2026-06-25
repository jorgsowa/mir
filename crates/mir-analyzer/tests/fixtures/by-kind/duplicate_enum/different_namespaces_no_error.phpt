===description===
Two enums with the same short name in different namespaces are distinct — no DuplicateEnum.
===file===
<?php
namespace App;
enum Status { case Active; }

namespace Other;
enum Status { case Inactive; }
===expect===
