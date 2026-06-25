===description===
DuplicateClass fires for a namespaced class declared twice in the same file.
===file===
<?php
namespace App;

class User {}

class User {}
===expect===
DuplicateClass@6:0-6:13: Class App\User has already been defined
