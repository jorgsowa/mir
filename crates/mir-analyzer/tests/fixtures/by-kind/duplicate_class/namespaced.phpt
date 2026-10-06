===description===
DuplicateClass fires for a namespaced class declared twice in the same file.
===file===
<?php
namespace App;

class User {}

class User {}
//<^^^^^^^^^^^^^ DuplicateClass: Class App\User has already been defined
