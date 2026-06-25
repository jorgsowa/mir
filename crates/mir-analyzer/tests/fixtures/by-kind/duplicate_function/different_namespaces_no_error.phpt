===description===
Two functions with the same short name in different namespaces are distinct — no DuplicateFunction.
===file===
<?php
namespace App;
function greet(): string { return 'hello'; }

namespace Other;
function greet(): string { return 'hi'; }
===expect===
